//! Instruction encodings compared against GNU as.
//!
//! Every case is an instruction given to the typed API together with its Intel-syntax text. The
//! test writes all texts to a file and `tests/emitter_golden.py` assembles them with GNU as,
//! giving a table of the expected encodings: `text<TAB>hex`, `text<TAB>hex<TAB>reloc
//! offset<TAB>addend` for an instruction with a `[rip + sym]` operand (the disp32 field is
//! zero, gas emits an R_X86_64_PC32 relocation), or `text<TAB>error` if gas rejects the line.
//! The table is kept in the test's scratch directory (`golden.txt`) for inspection.
//!
//! Needs `python3` and x86-64 binutils (`x86_64-linux-gnu-as`/`objdump`, or the host's on
//! x86-64 Linux); without them the test is skipped, as the writer tests are
//! (`MEMEASM_REQUIRE_TOOLS=1` makes that a failure).

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use super::x86;
use super::{Base, Mem, Op, Operand, Reg, Size};
use crate::backend::writers::tests::{host_x86_64, need, run_ok, scratch, tool};

fn reg_name(r: Reg) -> String {
    super::regs::name(r)
}

fn imm_text(v: i64) -> String {
    if v == i64::MIN {
        "0x8000000000000000".into()
    } else {
        v.to_string()
    }
}

fn operand_text(o: &Operand) -> String {
    match o {
        Operand::Reg(r) => reg_name(*r),
        Operand::Imm(v) => imm_text(*v),
        Operand::Mem(m) => {
            let size = match m.size {
                None => "",
                Some(Size::Byte) => "BYTE PTR ",
                Some(Size::Word) => "WORD PTR ",
                Some(Size::Dword) => "DWORD PTR ",
                Some(Size::Qword) => "QWORD PTR ",
            };
            let disp = |d: i64| match d {
                0 => String::new(),
                d if d < 0 => format!("-{}", d.unsigned_abs()),
                d => format!("+{d}"),
            };
            let inner = match &m.base {
                Base::Reg(r) => format!("{}{}", reg_name(*r), disp(m.disp)),
                Base::Rip(s) => format!("rip+{s}{}", disp(m.disp)),
                Base::Abs => m.disp.to_string(),
            };
            format!("{size}[{inner}]")
        }
    }
}

fn text(op: Op, ops: &[Operand]) -> String {
    let name = format!("{op:?}").to_lowercase();
    let args: Vec<String> = ops.iter().map(operand_text).collect();
    if args.is_empty() {
        name
    } else {
        format!("{name} {}", args.join(", "))
    }
}

fn all_regs() -> Vec<Reg> {
    let mut v = Vec::new();
    for n in 0..16 {
        v.extend([Reg::R64(n), Reg::R32(n), Reg::R16(n), Reg::R8(n)]);
    }
    v.extend((0..4).map(Reg::R8H));
    v
}

/// A partner register of the same kind for `r`.
fn partner(r: Reg) -> Reg {
    match r {
        Reg::R64(n) => Reg::R64((n + 9) % 16),
        Reg::R32(n) => Reg::R32((n + 9) % 16),
        Reg::R16(n) => Reg::R16((n + 9) % 16),
        Reg::R8(n) => Reg::R8((n + 3) % 16),
        Reg::R8H(n) => Reg::R8H((n + 1) % 4),
        Reg::Xmm(n) => Reg::Xmm((n + 1) % 16),
    }
}

/// One register of every size class (plus a REX-requiring and a high-byte one).
const SOME_REGS: [Reg; 11] = [
    Reg::R64(0), Reg::R64(9), Reg::R32(0), Reg::R32(4), Reg::R16(0), Reg::R16(13), Reg::R8(0),
    Reg::R8(6), Reg::R8(15), Reg::R8H(1), Reg::R8H(0),
];

const IMMS: [i64; 21] = [
    0, 1, -1, 127, 128, -128, -129, 255, 256, 0x7fff, 0x8000, -0x8001, 0xffff, 0x7fff_ffff,
    0x8000_0000, -0x8000_0000, 0xffff_ffff, 0x1_ffff_ffff, -0x8000_0001, i64::MAX, i64::MIN,
];

const SIZES: [Option<Size>; 5] = [None, Some(Size::Byte), Some(Size::Word), Some(Size::Dword), Some(Size::Qword)];

fn mem(size: Option<Size>, base: Base, disp: i64) -> Mem {
    Mem { size, base, disp }
}

/// Every base register (64 and 32 bit) without displacement, the special bases with edge
/// displacements, RIP-relative and absolute operands.
fn all_mems(size: Option<Size>) -> Vec<Mem> {
    let mut v = Vec::new();
    for n in 0..16 {
        v.push(mem(size, Base::Reg(Reg::R64(n)), 0));
        v.push(mem(size, Base::Reg(Reg::R32(n)), 0));
    }
    for b in [Reg::R64(0), Reg::R64(4), Reg::R64(5), Reg::R64(12), Reg::R64(13), Reg::R32(5)] {
        for d in [1, -1, 8, 127, 128, -128, -129, 0x7fff_ffff, -0x8000_0000] {
            v.push(mem(size, Base::Reg(b), d));
        }
    }
    for d in [0, 5, -5, 0x7fff_ffff] {
        v.push(mem(size, Base::Rip("sym".into()), d));
    }
    for d in [0, 69, -1, 0x7fff_ffff, -0x8000_0000] {
        v.push(mem(size, Base::Abs, d));
    }
    v
}

/// A few memory operands for the larger cross products.
fn some_mems(size: Option<Size>) -> Vec<Mem> {
    vec![
        mem(size, Base::Reg(Reg::R64(3)), 0),
        mem(size, Base::Reg(Reg::R64(12)), 0),
        mem(size, Base::Reg(Reg::R64(13)), 16),
        mem(size, Base::Reg(Reg::R32(0)), 0),
        mem(size, Base::Reg(Reg::R64(4)), 300),
        mem(size, Base::Rip("sym".into()), 0),
        mem(size, Base::Abs, 69),
    ]
}

fn cases() -> Vec<(Op, Vec<Operand>)> {
    use Operand as O;
    let mut c: Vec<(Op, Vec<Operand>)> = Vec::new();
    for op in [Op::Cqo, Op::Ret, Op::Nop, Op::Hlt, Op::Int3, Op::Syscall] {
        c.push((op, vec![]));
    }
    let regs = all_regs();
    let alu = [Op::Mov, Op::Add, Op::Sub, Op::And, Op::Xor, Op::Cmp, Op::Test];
    for op in alu {
        for &x in &regs {
            c.push((op, vec![O::Reg(x), O::Reg(partner(x))]));
            c.push((op, vec![O::Reg(x), O::Imm(100)]));
        }
        for &x in &SOME_REGS {
            for &v in &IMMS {
                c.push((op, vec![O::Reg(x), O::Imm(v)]));
            }
            let mut mems = some_mems(None);
            if op != Op::Mov {
                // (the other operations share the memory operand encoding with mov)
                mems = vec![mems[0].clone(), mems[3].clone(), mems[5].clone()];
            }
            mems.push(mem(Some(Size::Word), Base::Reg(Reg::R64(3)), 0));
            for m in mems {
                c.push((op, vec![O::Reg(x), O::Mem(m.clone())]));
                c.push((op, vec![O::Mem(m), O::Reg(x)]));
            }
        }
        for size in SIZES {
            for m in some_mems(size) {
                for v in [1, 1000, -129, 0xffff_ffff] {
                    c.push((op, vec![O::Mem(m.clone()), O::Imm(v)]));
                }
            }
        }
        // mixing a high-byte register with one that needs REX
        c.push((op, vec![O::Reg(Reg::R8H(0)), O::Reg(Reg::R8(6))]));
    }
    for m in all_mems(None) {
        c.push((Op::Mov, vec![O::Reg(Reg::R64(0)), O::Mem(m.clone())]));
        c.push((Op::Mov, vec![O::Mem(m.clone()), O::Reg(Reg::R8(15))]));
        c.push((Op::Mov, vec![O::Mem(m.clone()), O::Reg(Reg::R8H(2))]));
        c.push((Op::Lea, vec![O::Reg(Reg::R64(2)), O::Mem(m.clone())]));
        c.push((Op::Push, vec![O::Mem(m.clone())]));
        c.push((Op::Movups, vec![O::Reg(Reg::Xmm(0)), O::Mem(m.clone())]));
        c.push((Op::Movups, vec![O::Mem(m.clone()), O::Reg(Reg::Xmm(9))]));
    }
    for m in all_mems(Some(Size::Qword)) {
        c.push((Op::Add, vec![O::Mem(m.clone()), O::Imm(1)]));
        c.push((Op::Test, vec![O::Mem(m.clone()), O::Imm(1)]));
        c.push((Op::Idiv, vec![O::Mem(m)]));
    }
    for &x in &regs {
        for op in [Op::Not, Op::Push, Op::Pop, Op::Rdrand] {
            // (push/pop of 16-bit registers are valid, but not supported by the encoder)
            if !(matches!(op, Op::Push | Op::Pop) && matches!(x, Reg::R16(_))) {
                c.push((op, vec![O::Reg(x)]));
            }
        }
        for op in [Op::Shl, Op::Shr] {
            c.push((op, vec![O::Reg(x), O::Imm(1)]));
        }
        c.push((Op::Imul, vec![O::Reg(x), O::Reg(partner(x))]));
        c.push((Op::Imul, vec![O::Reg(x), O::Imm(100)]));
    }
    for size in SIZES {
        for m in some_mems(size) {
            for op in [Op::Not, Op::Idiv, Op::Pop] {
                if op == Op::Pop && size == Some(Size::Word) {
                    continue; // (valid, but not supported by the encoder)
                }
                c.push((op, vec![O::Mem(m.clone())]));
            }
            for op in [Op::Shl, Op::Shr] {
                c.push((op, vec![O::Mem(m.clone()), O::Imm(1)]));
            }
        }
    }
    for &x in &SOME_REGS {
        for &v in &IMMS {
            c.push((Op::Imul, vec![O::Reg(x), O::Imm(v)]));
        }
        for m in some_mems(None).into_iter().chain(some_mems(Some(Size::Dword))) {
            c.push((Op::Imul, vec![O::Reg(x), O::Mem(m)]));
        }
    }
    for &v in &IMMS {
        c.push((Op::Push, vec![O::Imm(v)]));
    }
    c
}

struct Expect {
    hex: String,
    /// `(offset, addend)` of the PC32 relocation
    reloc: Option<(usize, i64)>,
}

/// The Intel-syntax texts of all cases, without duplicates, in case order.
fn case_texts() -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    cases().iter().map(|(op, ops)| text(*op, ops)).filter(|t| seen.insert(t.clone())).collect()
}

/// The expected encoding of every case text, generated with GNU as (see the module docs).
/// `None` if the tools are missing.
fn golden() -> Option<HashMap<String, Option<Expect>>> {
    let python = need(tool(&["python3"]), "python3")?;
    let host = |name: &str| host_x86_64().then(|| tool(&[name])).flatten();
    let gas = need(tool(&["x86_64-linux-gnu-as"]).or_else(|| host("as")), "x86-64 GNU as")?;
    let objdump = need(tool(&["x86_64-linux-gnu-objdump"]).or_else(|| host("objdump")), "x86-64 objdump")?;
    let dir = scratch("emitter_golden");
    let cases_path = dir.join("cases.txt");
    std::fs::write(&cases_path, case_texts().join("\n") + "\n").unwrap();
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/emitter_golden.py");
    let out = run_ok(Command::new(python).arg("-I").arg(script).arg(&cases_path).env("AS", gas).env("OBJDUMP", objdump));
    let table = String::from_utf8(out.stdout).unwrap();
    std::fs::write(dir.join("golden.txt"), &table).unwrap();
    let mut g = HashMap::new();
    for line in table.lines() {
        let f: Vec<&str> = line.split('\t').collect();
        let e = match f[1] {
            "error" => None,
            hex => Some(Expect {
                hex: hex.to_string(),
                reloc: (f.len() == 4).then(|| (f[2].parse().unwrap(), f[3].parse().unwrap())),
            }),
        };
        assert!(g.insert(f[0].to_string(), e).is_none(), "duplicate golden line {line}");
    }
    Some(g)
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|b| format!("{b:02x}")).collect()
}

#[test]
fn encodings_match_gas() {
    let Some(g) = golden() else { return };
    let mut failures = Vec::new();
    let mut checked = 0;
    for (op, ops) in &cases() {
        let t = text(*op, ops);
        let want = g.get(&t).unwrap_or_else(|| panic!("`{t}' missing in gas's output"));
        let got = x86::encode(*op, ops)
            .ok()
            .map(|e| (hex(&e.bytes), e.rip.map(|(field, _, disp)| (field, disp - (e.bytes.len() - field) as i64))));
        let ok = match (want, &got) {
            (None, None) => true,
            (Some(w), Some((h, r))) => w.hex == *h && w.reloc == *r,
            _ => false,
        };
        if !ok {
            failures.push(format!("{t}: gas {:?}, emitter {got:?}", want.as_ref().map(|w| (&w.hex, w.reloc))));
        }
        checked += 1;
    }
    assert!(failures.is_empty(), "{} mismatches:\n{}", failures.len(), failures.join("\n"));
    assert!(g.len() > 2000 && g.values().any(Option::is_none), "only {} cases ({checked} checked)", g.len());
}

/// The writers' extra forms (`call`/`jmp` r/m64, indexed `lea`), checked against GNU as.
#[test]
fn writer_stub_forms_match_gas() {
    use x86::BranchKind::{Call, Jcc, Jmp};
    use Reg::R64;
    let rip = Operand::Mem(mem(None, Base::Rip("sym".into()), 0));
    let indirect = [
        (Call, Operand::Reg(R64(0)), "ffd0"),
        (Call, Operand::Reg(R64(9)), "41ffd1"),
        (Jmp, Operand::Reg(R64(15)), "41ffe7"),
        (Call, rip.clone(), "ff1500000000"),
        (Jmp, rip.clone(), "ff2500000000"),
        (Call, Operand::Mem(mem(Some(Size::Qword), Base::Reg(R64(4)), 8)), "ff542408"),
        (Jmp, Operand::Mem(mem(None, Base::Reg(R64(13)), 0)), "41ff6500"),
    ];
    for (kind, target, want) in indirect {
        let e = x86::encode_indirect(kind, &target).unwrap();
        assert_eq!(hex(&e.bytes), want, "{kind:?} {target:?}");
        assert_eq!(e.rip.map(|(f, ..)| f), (target == rip).then_some(2));
    }
    assert!(x86::encode_indirect(Jcc(super::Cond::E), &Operand::Reg(R64(0))).is_err());
    assert!(x86::encode_indirect(Call, &Operand::Reg(Reg::R32(0))).is_err());
    assert!(x86::encode_indirect(Call, &Operand::Mem(mem(Some(Size::Dword), Base::Reg(R64(0)), 0))).is_err());

    let lea = [
        ((2, 6, 7, 8, 8), "488d54fe08"),
        ((12, 13, 14, 1, 0), "4f8d643500"),
        ((0, 5, 1, 2, 0x1000), "488d844d00100000"),
        ((0, 3, 1, 4, 0), "488d048b"),
        ((8, 12, 9, 2, -1), "4f8d444cff"),
    ];
    for ((d, b, i, s, disp), want) in lea {
        let e = x86::encode_lea_indexed(R64(d), R64(b), R64(i), s, disp).unwrap();
        assert_eq!(hex(&e.bytes), want);
        assert!(e.rip.is_none());
    }
    assert!(x86::encode_lea_indexed(R64(0), R64(0), R64(4), 1, 0).is_err());
    assert!(x86::encode_lea_indexed(R64(0), R64(0), R64(1), 3, 0).is_err());
    assert!(x86::encode_lea_indexed(Reg::R32(0), R64(0), R64(1), 1, 0).is_err());
}
