//! Instruction encodings compared against a golden table produced by GNU as
//! (`tests/emitter_golden.txt`).
//!
//! Every case is an instruction given to the typed API together with its Intel-syntax text. To
//! regenerate the table after changing the cases (needs `x86_64-linux-gnu-as` and `objdump`):
//!
//! ```sh
//! EMITTER_GOLDEN_CASES=/tmp/cases.txt cargo test write_golden_cases -- --ignored
//! python3 tests/emitter_golden.py /tmp/cases.txt > tests/emitter_golden.txt
//! ```
//!
//! Each line of the table is `text<TAB>hex`, `text<TAB>hex<TAB>reloc offset<TAB>addend` for an
//! instruction with a `[rip + sym]` operand (the disp32 field is zero, gas emits an
//! R_X86_64_PC32 relocation), or `text<TAB>error` if gas rejects the line.

use std::collections::HashMap;

use super::x86;
use super::{Base, Mem, Op, Operand, Reg, Size};

const GOLDEN: &str = include_str!("../../../tests/emitter_golden.txt");

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

fn golden() -> HashMap<String, Option<Expect>> {
    let mut g = HashMap::new();
    for line in GOLDEN.lines() {
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
    g
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|b| format!("{b:02x}")).collect()
}

/// The encoder under test: `(hex, Some((reloc offset, addend)))`, or `None` if rejected.
pub(super) type Encoder = fn(Op, &[Operand]) -> Option<(String, Option<(usize, i64)>)>;

fn new_encoder(op: Op, ops: &[Operand]) -> Option<(String, Option<(usize, i64)>)> {
    let e = x86::encode(op, ops).ok()?;
    let reloc = e.rip.map(|(field, _, disp)| (field, disp - (e.bytes.len() - field) as i64));
    Some((hex(&e.bytes), reloc))
}

pub(super) fn check_against_golden(enc: Encoder) -> usize {
    let g = golden();
    let cases = cases();
    let mut seen = std::collections::HashSet::new();
    let mut failures = Vec::new();
    for (op, ops) in &cases {
        let t = text(*op, ops);
        if !seen.insert(t.clone()) {
            continue;
        }
        let want = g.get(&t).unwrap_or_else(|| panic!("`{t}' missing in the golden table (regenerate it)"));
        let got = enc(*op, ops);
        let ok = match (want, &got) {
            (None, None) => true,
            (Some(w), Some((h, r))) => w.hex == *h && w.reloc == *r,
            _ => false,
        };
        if !ok {
            failures.push(format!("{t}: gas {:?}, emitter {got:?}", want.as_ref().map(|w| (&w.hex, w.reloc))));
        }
    }
    assert_eq!(seen.len(), g.len(), "the golden table has lines without a case (regenerate it)");
    assert!(failures.is_empty(), "{} mismatches:\n{}", failures.len(), failures.join("\n"));
    seen.len()
}

#[test]
fn encodings_match_gas() {
    let n = check_against_golden(new_encoder);
    assert!(n > 2000);
}

/// Writes the Intel-syntax text of every case to `$EMITTER_GOLDEN_CASES` (see the module docs).
#[test]
#[ignore]
fn write_golden_cases() {
    let path = std::env::var("EMITTER_GOLDEN_CASES").expect("EMITTER_GOLDEN_CASES not set");
    let mut out = String::new();
    let mut seen = std::collections::HashSet::new();
    for (op, ops) in cases() {
        let t = text(op, &ops);
        if seen.insert(t.clone()) {
            out.push_str(&t);
            out.push('\n');
        }
    }
    std::fs::write(path, out).unwrap();
}
