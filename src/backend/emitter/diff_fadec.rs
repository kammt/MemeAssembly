//! Transitional differential test: the pure-Rust encoder (`x86.rs`) against the fadec-based
//! encoder it replaces (`encode.rs`), on an exhaustive enumeration of operand combinations.
//! Run with `cargo test --release diff_fadec -- --nocapture`.

use super::layout::BranchKind as OldKind;
use super::x86::{self, BranchKind};
use super::*;
use crate::backend::fadec;

fn all_regs() -> Vec<Reg> {
    let mut v = Vec::new();
    for n in 0..16 {
        v.extend([Reg::R64(n), Reg::R32(n), Reg::R16(n), Reg::R8(n)]);
    }
    v.extend((0..4).map(Reg::R8H));
    v
}

const DISPS: [i64; 13] = [
    0, 1, -1, 8, 16, 127, 128, -128, -129, 0x7fff_ffff, -0x8000_0000, 0x8000_0000, 0x1_0000_0000,
];
const SIZES: [Option<Size>; 5] = [None, Some(Size::Byte), Some(Size::Word), Some(Size::Dword), Some(Size::Qword)];

fn mems(disps: &[i64], sizes: &[Option<Size>]) -> Vec<Mem> {
    let mut bases: Vec<Base> = Vec::new();
    for n in 0..16 {
        bases.push(Base::Reg(Reg::R64(n)));
        bases.push(Base::Reg(Reg::R32(n)));
    }
    bases.extend([Base::Reg(Reg::R16(0)), Base::Reg(Reg::R8(3)), Base::Rip("sym".into()), Base::Abs]);
    let mut v = Vec::new();
    for b in &bases {
        for &disp in disps {
            for &size in sizes {
                v.push(Mem { size, base: b.clone(), disp });
            }
        }
    }
    v
}

const IMMS: [i64; 30] = [
    0, 1, -1, 2, 25, 127, 128, -128, -129, 255, 256, 9000, 0x7fff, 0x8000, -0x8000, -0x8001, 0xffff,
    0x1_0000, 0x7fff_ffff, 0x8000_0000, -0x8000_0000, -0x8000_0001, 0xffff_ffff, 0x1_0000_0000,
    0x1_ffff_ffff, 0x1_0000_0001, -0x1_0000_0001, i64::MAX, i64::MIN, 0x200_002e,
];

/// Forms the old encoder accepted that the new one deliberately drops: the translator never
/// produces them (see `src/frontend/encoder.rs`).
fn dropped(op: Op, ops: &[Operand]) -> bool {
    use Operand as O;
    match (op, ops) {
        // push/pop of 16-bit registers or WORD PTR memory (the frontend pushes 64-bit values)
        (Op::Push | Op::Pop, [O::Reg(Reg::R16(_))]) => true,
        (Op::Push | Op::Pop, [O::Mem(m)]) => m.size == Some(Size::Word),
        // shifts by counts other than 1 or by cl, one-operand imul, inc/dec, movups xmm, xmm
        (Op::Shl | Op::Shr, [_, O::Imm(v)]) => *v != 1,
        (Op::Shl | Op::Shr, [_, O::Reg(_)]) => true,
        (Op::Imul, [_]) | (Op::Inc | Op::Dec, _) => true,
        (Op::Movups, [O::Reg(_), O::Reg(_)]) => true,
        _ => false,
    }
}

#[derive(Default)]
struct Stats {
    cases: usize,
    both_ok: usize,
    both_err: usize,
    dropped: usize,
}

fn check(st: &mut Stats, op: Op, ops: &[Operand]) {
    st.cases += 1;
    let old = encode::encode(op, ops);
    let new = x86::encode(op, ops);
    match (old, new) {
        (Ok(a), Ok(b)) => {
            assert_eq!((&a.bytes, &a.rip), (&b.bytes, &b.rip), "{op:?} {ops:?}");
            st.both_ok += 1;
        }
        (Err(_), Err(_)) => st.both_err += 1,
        (Ok(_), Err(_)) if dropped(op, ops) => st.dropped += 1,
        (a, b) => panic!(
            "{op:?} {ops:?}: old {:?} new {:?}",
            a.map(|e| e.bytes),
            b.map(|e| e.bytes)
        ),
    }
}

#[test]
fn diff_fadec_instructions() {
    use Operand as O;
    let regs = all_regs();
    let xmms: Vec<Reg> = (0..16).map(Reg::Xmm).collect();
    let all_mems = mems(&DISPS, &SIZES);
    let st = &mut Stats::default();

    for op in [Op::Cqo, Op::Ret, Op::Nop, Op::Hlt, Op::Int3, Op::Syscall] {
        check(st, op, &[]);
    }
    for op in [Op::Mov, Op::Add, Op::Sub, Op::And, Op::Xor, Op::Cmp, Op::Test, Op::Imul] {
        for &x in &regs {
            for &y in &regs {
                check(st, op, &[O::Reg(x), O::Reg(y)]);
            }
            for &v in &IMMS {
                check(st, op, &[O::Reg(x), O::Imm(v)]);
            }
            for m in &all_mems {
                check(st, op, &[O::Reg(x), O::Mem(m.clone())]);
                check(st, op, &[O::Mem(m.clone()), O::Reg(x)]);
            }
        }
        for m in &all_mems {
            for &v in &IMMS {
                check(st, op, &[O::Mem(m.clone()), O::Imm(v)]);
            }
        }
    }
    for op in [Op::Not, Op::Idiv, Op::Push, Op::Pop, Op::Rdrand, Op::Imul, Op::Inc, Op::Dec] {
        for &x in &regs {
            check(st, op, &[O::Reg(x)]);
        }
        for m in &all_mems {
            check(st, op, &[O::Mem(m.clone())]);
        }
    }
    for &v in &IMMS {
        check(st, Op::Push, &[O::Imm(v)]);
    }
    for op in [Op::Shl, Op::Shr] {
        let counts = [O::Imm(1), O::Imm(0), O::Imm(2), O::Imm(0x101), O::Reg(Reg::R8(1))];
        for c in &counts {
            for &x in &regs {
                check(st, op, &[O::Reg(x), c.clone()]);
            }
            for m in &all_mems {
                check(st, op, &[O::Mem(m.clone()), c.clone()]);
            }
        }
    }
    for &x in regs.iter().chain(&xmms) {
        for m in &all_mems {
            check(st, Op::Lea, &[O::Reg(x), O::Mem(m.clone())]);
            check(st, Op::Movups, &[O::Reg(x), O::Mem(m.clone())]);
            check(st, Op::Movups, &[O::Mem(m.clone()), O::Reg(x)]);
        }
        for &y in &xmms {
            check(st, Op::Movups, &[O::Reg(x), O::Reg(y)]);
        }
    }
    eprintln!(
        "diff_fadec_instructions: {} cases: {} identical encodings, {} rejected by both, {} dropped forms",
        st.cases, st.both_ok, st.both_err, st.dropped
    );
    assert!(st.both_ok > 100_000);
}

#[test]
fn diff_fadec_branches() {
    let conds = [Cond::E, Cond::Ne, Cond::Z, Cond::Nz, Cond::G, Cond::L, Cond::Ge, Cond::Le, Cond::A, Cond::B];
    let mut kinds = vec![(BranchKind::Jmp, OldKind::Jmp), (BranchKind::Call, OldKind::Call)];
    kinds.extend(conds.iter().map(|&c| (BranchKind::Jcc(c), OldKind::Jcc(c))));
    let mut n = 0;
    for (new, old) in kinds {
        for long in [false, true] {
            if new == BranchKind::Call && !long {
                continue;
            }
            let rels: &[i64] = if long {
                &[i32::MIN as i64, -0x8000, -129, -128, -1, 0, 1, 127, 128, 0x8000, i32::MAX as i64]
            } else {
                &[-128, -127, -2, -1, 0, 1, 2, 126, 127]
            };
            let m = old.fadec_mnemonic() | if long { fadec::FE_JMPL } else { 0 };
            for &rel in rels {
                let at: u64 = 0x1_0000_0000;
                let target = (at as i64 + new.len(long) as i64 + rel) as u64;
                let a = fadec::encode_branch(m, at, target).unwrap();
                assert_eq!(a, x86::branch(new, long, rel as i32), "{new:?} long={long} rel={rel}");
                n += 1;
            }
            // unresolved (relocated) form: rel32 field zero
            let mut a = fadec::encode_branch(m, 0, 0).unwrap();
            let len = a.len();
            if long {
                a[len - 4..].fill(0);
                assert_eq!(a, x86::branch(new, true, 0));
            }
            n += 1;
        }
    }
    eprintln!("diff_fadec_branches: {n} cases identical");
}

/// The old (fadec) encoder against the gas golden table, too.
#[test]
fn diff_fadec_golden() {
    fn old(op: Op, ops: &[Operand]) -> Option<(String, Option<(usize, i64)>)> {
        let e = encode::encode(op, ops).ok()?;
        let reloc = e.rip.map(|(field, _, disp)| (field, disp - (e.bytes.len() - field) as i64));
        Some((e.bytes.iter().map(|b| format!("{b:02x}")).collect(), reloc))
    }
    super::x86_tests::check_against_golden(old);
}
