//! Emitter tests on the typed API: layout (branch relaxation, labels, alignment), relocations
//! per object format and debug line rows. Instruction encodings are checked against the
//! gas-generated table in `x86_tests`. Expected bytes are from GNU as 2.46, and for Mach-O from
//! `llvm-mc -triple x86_64-apple-macos`.

use super::*;
use crate::backend::module::RelocKind;

use BranchTarget::Local;
use Operand::{Imm, Reg as R};
use Reg::{R64, R8, R8H};

fn hex(s: &str) -> Vec<u8> {
    let s: String = s.split_whitespace().collect();
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}

fn sym(s: &str) -> BranchTarget {
    BranchTarget::Symbol(s.into())
}

fn mem(size: Option<Size>, base: Base, disp: i64) -> Operand {
    Operand::Mem(Mem { size, base, disp })
}

fn nops(em: &mut Emitter, n: usize) {
    for _ in 0..n {
        em.insn(Op::Nop, &[]).unwrap();
    }
}

fn text(em: Emitter) -> Vec<u8> {
    em.finish().unwrap().sections[0].data.clone()
}

/// `(offset, kind, symbol, addend)` of the `.text` relocations.
fn text_relocs(m: &ObjModule) -> Vec<(u64, RelocKind, String, i64)> {
    m.sections[0].relocs.iter().map(|r| (r.offset, r.kind, m.symbols[r.symbol].name.clone(), r.addend)).collect()
}

#[test]
fn memory_operand_forms() {
    let mut em = Emitter::new(Target::Linux);
    em.insn(Op::Mov, &[R(R64(0)), mem(None, Base::Abs, 69)]).unwrap();
    em.insn(Op::Mov, &[R(R64(0)), mem(None, Base::Reg(Reg::R32(0)), 0)]).unwrap();
    em.insn(Op::Mov, &[mem(Some(Size::Byte), Base::Reg(R64(12)), 0), Imm(1)]).unwrap();
    em.insn(Op::Lea, &[R(R64(1)), mem(None, Base::Reg(R64(13)), -8)]).unwrap();
    assert_eq!(text(em), hex("48 8b 04 25 45 00 00 00  67 48 8b 00  41 c6 04 24 01  49 8d 4d f8"));
}

#[test]
fn rip_relative_and_relocs() {
    let mut em = Emitter::new(Target::Linux);
    em.section(SectionKind::Data);
    em.label(".Lc").unwrap();
    em.bytes(b"a");
    em.section(SectionKind::Text);
    em.insn(Op::Mov, &[mem(Some(Size::Byte), Base::Rip(".Lc".into()), 0), Imm(0x61)]).unwrap();
    em.call(sym("foo")).unwrap();
    // `[rip + $]` refers to the start of the instruction.
    em.insn(Op::Lea, &[R(R64(0)), mem(None, Base::Rip("$".into()), 1)]).unwrap();
    let m = em.finish().unwrap();
    let t = &m.sections[0];
    assert_eq!(t.data, hex("c6 05 00000000 61  e8 00000000  48 8d 05 faffffff"));
    assert_eq!(
        text_relocs(&m),
        [(2, RelocKind::Pc32, ".Lc".into(), -5), (8, RelocKind::Branch32, "foo".into(), -4)]
    );
    // the `.data` symbol is referenced through the section (temporary label)
    assert!(m.symbols.iter().any(|s| s.name == ".Lc" && s.temporary));
}

#[test]
fn relaxation_boundaries() {
    // backward: rel8 reaches -128
    for (n, want) in [(126, "eb 80"), (127, "e9 7c ff ff ff")] {
        let mut em = Emitter::new(Target::Linux);
        em.label("a").unwrap();
        nops(&mut em, n);
        em.jump(None, sym("a")).unwrap();
        assert_eq!(&text(em)[n..], &hex(want)[..], "{n}");
    }
    // forward: rel8 reaches +127; jcc grows to 6 bytes
    for (n, cond, want) in [
        (127, None, "eb 7f"),
        (128, None, "e9 80 00 00 00"),
        (127, Some(Cond::Ge), "7d 7f"),
        (128, Some(Cond::Ge), "0f 8d 80 00 00 00"),
        (127, Some(Cond::B), "72 7f"),
        (128, Some(Cond::B), "0f 82 80 00 00 00"),
        (127, Some(Cond::Ae), "73 7f"),
        (128, Some(Cond::Ae), "0f 83 80 00 00 00"),
        (127, Some(Cond::Ne), "75 7f"),
        (128, Some(Cond::Ne), "0f 85 80 00 00 00"),
    ] {
        let mut em = Emitter::new(Target::Linux);
        em.jump(cond, sym("b")).unwrap();
        nops(&mut em, n);
        em.label("b").unwrap();
        let t = text(em);
        assert_eq!(&t[..t.len() - n], &hex(want)[..], "{n} {cond:?}");
    }
    // A long jump moves the label: the earlier jump over both needs rel32 too.
    let mut em = Emitter::new(Target::Linux);
    em.jump(Some(Cond::E), sym("end")).unwrap();
    em.jump(None, sym("z")).unwrap();
    nops(&mut em, 123);
    em.label("end").unwrap();
    nops(&mut em, 10);
    em.label("z").unwrap();
    let t = text(em);
    // jmp z: 133 bytes → rel32 (5 bytes); je end: 5 + 123 = 128 → rel32 as well
    assert_eq!(&t[..11], &hex("0f 84 80 00 00 00 e9 85 00 00 00")[..]);
}

#[test]
fn relaxation_long_backward() {
    let mut em = Emitter::new(Target::Linux);
    em.local_label(1);
    em.insn(Op::Xor, &[R(R64(0)), R(R64(0))]).unwrap();
    for _ in 0..40 {
        em.insn(Op::Mov, &[R(R64(0)), Imm(1)]).unwrap();
    }
    em.jump(None, Local { n: 1, forward: false }).unwrap();
    em.jump(Some(Cond::Z), Local { n: 1, forward: false }).unwrap();
    let t = text(em);
    // 3 + 40*7 = 283 bytes before the jmp: rel32 needed.
    assert_eq!(&t[283..], &hex("e9 e0 fe ff ff 0f 84 da fe ff ff")[..]);
}

#[test]
fn local_labels() {
    let mut em = Emitter::new(Target::Linux);
    em.local_label(1);
    em.jump(None, Local { n: 1, forward: true }).unwrap();
    em.jump(None, Local { n: 1, forward: false }).unwrap();
    em.local_label(1);
    em.jump(None, Local { n: 1, forward: false }).unwrap();
    assert_eq!(text(em), hex("eb 02 eb fc eb fe"));

    let mut em = Emitter::new(Target::Linux);
    assert_eq!(
        em.jump(None, Local { n: 3, forward: false }).unwrap_err(),
        "backward ref to unknown label \"3:\""
    );
    em.jump(None, Local { n: 2, forward: true }).unwrap();
    assert_eq!(
        em.finish().unwrap_err(),
        "local label `\"2\" (instance number 1 of a fb label)' is not defined"
    );
}

#[test]
fn branch_targets() {
    // Symbol names are taken verbatim: `$` and `rax` are ordinary symbols here (the -S text
    // writes them quoted).
    let mut em = Emitter::new(Target::Linux);
    em.call(sym("$")).unwrap();
    em.call(sym("rax")).unwrap();
    assert_eq!(em.call(sym("")).unwrap_err(), "internal error: invalid symbol name `'");
    assert_eq!(em.jump(None, sym("a b")).unwrap_err(), "internal error: invalid symbol name `a b'");
    let m = em.finish().unwrap();
    assert_eq!(m.sections[0].data, hex("e8 00000000 e8 00000000"));
    assert_eq!(
        text_relocs(&m),
        [(1, RelocKind::Branch32, "$".into(), -4), (6, RelocKind::Branch32, "rax".into(), -4)]
    );
}

#[test]
fn errors() {
    let mut em = Emitter::new(Target::Linux);
    em.label("x").unwrap();
    em.label("x").unwrap(); // same position: tolerated like gas
    em.insn(Op::Nop, &[]).unwrap();
    assert_eq!(em.label("x").unwrap_err(), "symbol `x' is already defined");
    assert_eq!(em.label(".text").unwrap_err(), "symbol `.text' is already defined");
    assert_eq!(
        em.insn(Op::Mov, &[R(R8H(0)), R(R8(6))]).unwrap_err(),
        "internal error: unsupported operand combination for `mov'"
    );
    assert_eq!(
        em.insn(Op::Shl, &[R(R64(0)), Imm(2)]).unwrap_err(),
        "internal error: unsupported operand combination for `shl'"
    );
    assert_eq!(
        em.insn(Op::Add, &[mem(None, Base::Reg(R64(0)), 0), Imm(1)]).unwrap_err(),
        "internal error: unsupported operand combination for `add'"
    );
    // On Windows, section names are ordinary symbols.
    let mut em = Emitter::new(Target::Windows);
    em.label(".text").unwrap();
}

#[test]
fn label_names() {
    // Label names are taken verbatim (no gas character-constant scrubbing).
    let mut em = Emitter::new(Target::Linux);
    em.label(".L97Wins_0").unwrap();
    em.jump(Some(Cond::G), sym(".L97Wins_0")).unwrap();
    assert_eq!(em.label(".L'a'Wins_0").unwrap_err(), "internal error: invalid label name `.L'a'Wins_0'");
    assert_eq!(em.label("a b").unwrap_err(), "internal error: invalid label name `a b'");
    assert_eq!(em.label("1x").unwrap_err(), "internal error: invalid label name `1x'");
    let m = em.finish().unwrap();
    assert_eq!(m.sections[0].data, hex("7f fe"));
}

#[test]
fn alignment_padding() {
    // `.text` is padded with single-byte NOPs, `.data` with zeros.
    let mut em = Emitter::new(Target::Linux);
    em.insn(Op::Ret, &[]).unwrap();
    em.align(16).unwrap();
    em.section(SectionKind::Data);
    em.bytes(&[1]);
    em.align(8).unwrap();
    em.bytes(&[2]);
    assert_eq!(em.align(3).unwrap_err(), "alignment not a power of 2");
    let m = em.finish().unwrap();
    let mut want = hex("c3");
    want.resize(16, 0x90);
    assert_eq!(m.sections[0].data, want);
    assert_eq!(m.sections[0].align, 16);
    assert_eq!(m.sections[1].data, hex("01 00 00 00 00 00 00 00 02"));
    assert_eq!(m.sections[1].align, 8);
}

#[test]
fn coff_alignment() {
    // gas pads COFF .text/.data to 16 bytes and can't represent alignments above 8192.
    let mut em = Emitter::new(Target::Windows);
    em.insn(Op::Ret, &[]).unwrap();
    em.align(8192).unwrap();
    let m = em.finish().unwrap();
    let mut want = hex("c3");
    want.resize(8192, 0x90);
    assert_eq!(m.sections[0].data, want);
    assert_eq!(m.sections[0].align, 8192);

    let mut em = Emitter::new(Target::Windows);
    em.insn(Op::Ret, &[]).unwrap();
    let m = em.finish().unwrap();
    assert_eq!(m.sections[0].data, hex("c3 90 90 90 90 90 90 90 90 90 90 90 90 90 90 90"));

    let mut em = Emitter::new(Target::Windows);
    em.insn(Op::Ret, &[]).unwrap();
    em.align(16384).unwrap();
    assert_eq!(
        em.finish().unwrap_err(),
        "section .text: alignment 2**14 not representable\nnonrepresentable section on output"
    );
}

#[test]
fn macho_align() {
    // `align` takes a byte count on every target (the -S text writes `.p2align` for Mach-O).
    let mut em = Emitter::new(Target::MacOS);
    em.insn(Op::Nop, &[]).unwrap();
    em.align(32).unwrap();
    em.insn(Op::Ret, &[]).unwrap();
    let m = em.finish().unwrap();
    let mut want = vec![0x90; 32];
    want.push(0xc3);
    assert_eq!(m.sections[0].data, want);
    assert_eq!(m.sections[0].align, 32);
}

/// `.global gf; gf: ret; lf: ret; start: call gf; call lf; jmp gf; lea rax, [rip + gf];
/// lea rax, [rip + lf]; jmp start; jmp .Lx; .Lx: jz Lx; Lx: ret`
fn reloc_module(target: Target) -> ObjModule {
    let mut em = Emitter::new(target);
    em.global("gf");
    em.label("gf").unwrap();
    em.insn(Op::Ret, &[]).unwrap();
    em.label("lf").unwrap();
    em.insn(Op::Ret, &[]).unwrap();
    em.label("start").unwrap();
    em.call(sym("gf")).unwrap();
    em.call(sym("lf")).unwrap();
    em.jump(None, sym("gf")).unwrap();
    em.insn(Op::Lea, &[R(R64(0)), mem(None, Base::Rip("gf".into()), 0)]).unwrap();
    em.insn(Op::Lea, &[R(R64(0)), mem(None, Base::Rip("lf".into()), 0)]).unwrap();
    em.jump(None, sym("start")).unwrap();
    em.jump(None, sym(".Lx")).unwrap();
    em.label(".Lx").unwrap();
    em.jump(Some(Cond::Z), sym("Lx")).unwrap();
    em.label("Lx").unwrap();
    em.insn(Op::Ret, &[]).unwrap();
    em.finish().unwrap()
}

#[test]
fn relocations_elf() {
    // gas: calls and RIP references to global symbols keep their relocation (interposition);
    // `jmp`/`jcc` to them and references to local symbols are resolved.
    let m = reloc_module(Target::Linux);
    assert_eq!(
        m.sections[0].data,
        hex("c3 c3 e8 00000000 e8 f5ffffff eb f2 488d05 00000000 488d05 e5ffffff eb e4 eb 00 74 00 c3")
    );
    assert_eq!(
        text_relocs(&m),
        [(3, RelocKind::Branch32, "gf".into(), -4), (17, RelocKind::Pc32, "gf".into(), -4)]
    );
}

#[test]
fn relocations_coff() {
    let m = reloc_module(Target::Windows);
    assert_eq!(
        &m.sections[0].data[..35],
        &hex("c3 c3 e8 f9ffffff e8 f5ffffff eb f2 488d05 ebffffff 488d05 e5ffffff eb e4 eb 00 74 00 c3")[..]
    );
    assert!(m.sections[0].relocs.is_empty());
    // `L` labels are temporary for COFF as well.
    assert!(m.symbols.iter().any(|s| s.name == "Lx" && s.temporary));
}

#[test]
fn relocations_macho() {
    // LLVM: only references within one atom (from a non-temporary symbol to the next) are
    // resolved; `.L` labels are ordinary symbols, `L` labels are temporary.
    let m = reloc_module(Target::MacOS);
    assert_eq!(
        m.sections[0].data,
        hex("c3 c3 e8 00000000 e8 00000000 e9 00000000 488d05 00000000 488d05 00000000 eb e1 e9 00000000 \
             74 00 c3")
    );
    assert_eq!(
        text_relocs(&m),
        [
            (3, RelocKind::Branch32, "gf".into(), -4),
            (8, RelocKind::Branch32, "lf".into(), -4),
            (13, RelocKind::Branch32, "gf".into(), -4),
            (20, RelocKind::Pc32, "gf".into(), -4),
            (27, RelocKind::Pc32, "lf".into(), -4),
            (34, RelocKind::Branch32, ".Lx".into(), -4),
        ]
    );
    assert!(m.symbols.iter().any(|s| s.name == ".Lx" && !s.temporary));
    assert!(m.symbols.iter().any(|s| s.name == "Lx" && s.temporary));
}

#[test]
fn macho_atom_boundaries() {
    // A temporary label defined right before a non-temporary one at the same position belongs
    // to the previous atom; the instructions after them to the new one. (The writer refers to
    // `Ltmp` relative to its atom: llvm-mc emits `e9 01000000` with a BRANCH relocation against `a`.)
    let mut em = Emitter::new(Target::MacOS);
    em.label("a").unwrap();
    em.insn(Op::Nop, &[]).unwrap();
    em.label("Ltmp").unwrap();
    em.label("b").unwrap();
    em.jump(None, sym("Ltmp")).unwrap();
    em.jump(None, sym("b")).unwrap();
    let m = em.finish().unwrap();
    assert_eq!(m.sections[0].data, hex("90 e9 00000000 eb f9"));
    assert_eq!(text_relocs(&m), [(2, RelocKind::Branch32, "Ltmp".into(), -4)]);
}

#[test]
fn debug_line_rows() {
    // Like gas: every `.loc` keeps its row (consecutive ones at the same address, a pending one
    // flushed at the current address by the next), rows only start at instructions, line 0 and
    // a trailing `.loc` produce none. Paths are raw bytes.
    let mut em = Emitter::new(Target::Linux);
    em.set_debug_meta(b"/w\xff", "p");
    let f = em.debug_file(b"/x/\xfe.memeasm");
    em.debug_line(f, 1);
    em.label("a").unwrap();
    em.debug_line(f, 2);
    em.insn(Op::Nop, &[]).unwrap();
    em.debug_line(f, 0);
    em.insn(Op::Ret, &[]).unwrap();
    em.debug_line(f, 7);
    em.bytes(&[0x90]);
    em.debug_line(f, 8);
    em.bytes(&[0x90]);
    em.insn(Op::Nop, &[]).unwrap();
    em.debug_line(f, 9);
    let m = em.finish().unwrap();
    let d = m.debug.unwrap();
    let rows: Vec<(u64, u32)> = d.lines.iter().map(|r| (r.offset, r.line)).collect();
    assert_eq!(rows, [(0, 1), (0, 2), (3, 7), (4, 8)]);
    assert_eq!(d.files, [b"/x/\xfe.memeasm".to_vec()]);
    assert_eq!(d.comp_dir, b"/w\xff");
}
