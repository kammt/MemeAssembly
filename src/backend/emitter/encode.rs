//! Typed instruction → machine code, choosing the same forms (and accepting/rejecting the same
//! operand combinations) as GNU as does for the equivalent Intel-syntax line.

use super::regs;
use super::{Base, Mem, Op, Operand, Reg, Size};
use crate::backend::fadec::{self, *};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Sz {
    B,
    W,
    D,
    Q,
    X,
}

impl Sz {
    fn of_size(s: Size) -> Sz {
        match s {
            Size::Byte => Sz::B,
            Size::Word => Sz::W,
            Size::Dword => Sz::D,
            Size::Qword => Sz::Q,
            Size::Xmmword => Sz::X,
        }
    }
    fn idx(self) -> usize {
        match self {
            Sz::B => 0,
            Sz::W => 1,
            Sz::D => 2,
            Sz::Q => 3,
            Sz::X => 4,
        }
    }
}

/// An encoded instruction plus the location of its RIP-relative displacement, if any.
pub(super) struct Encoded {
    pub bytes: Vec<u8>,
    /// `(offset of disp32 field, symbol, displacement)`: the field must hold
    /// `symbol + displacement - end_of_instruction`.
    pub rip: Option<(usize, String, i64)>,
}

pub(super) fn mnemonic(op: Op) -> &'static str {
    match op {
        Op::Mov => "mov",
        Op::Movups => "movups",
        Op::Lea => "lea",
        Op::Push => "push",
        Op::Pop => "pop",
        Op::Add => "add",
        Op::Sub => "sub",
        Op::And => "and",
        Op::Xor => "xor",
        Op::Cmp => "cmp",
        Op::Test => "test",
        Op::Not => "not",
        Op::Inc => "inc",
        Op::Dec => "dec",
        Op::Shl => "shl",
        Op::Shr => "shr",
        Op::Imul => "imul",
        Op::Idiv => "idiv",
        Op::Cqo => "cqo",
        Op::Ret => "ret",
        Op::Nop => "nop",
        Op::Hlt => "hlt",
        Op::Int3 => "int3",
        Op::Syscall => "syscall",
        Op::Rdrand => "rdrand",
    }
}

fn reg_sz(r: Reg) -> Sz {
    match r {
        Reg::R64(_) => Sz::Q,
        Reg::R32(_) => Sz::D,
        Reg::R16(_) => Sz::W,
        Reg::R8(_) | Reg::R8H(_) => Sz::B,
        Reg::Xmm(_) => Sz::X,
    }
}

fn reg_op(r: Reg) -> FeOp {
    match r {
        Reg::R64(n) | Reg::R32(n) | Reg::R16(n) | Reg::R8(n) => gp(n),
        Reg::R8H(n) => FE_AH + n as FeOp,
        Reg::Xmm(n) => FE_XMM0 + n as FeOp,
    }
}

fn reg_needs_rex(r: Reg) -> bool {
    match r {
        Reg::R64(n) | Reg::R32(n) | Reg::R16(n) | Reg::Xmm(n) => n >= 8,
        Reg::R8(n) => n >= 4,
        Reg::R8H(_) => false,
    }
}

fn valid_reg(r: Reg) -> bool {
    match r {
        Reg::R64(n) | Reg::R32(n) | Reg::R16(n) | Reg::R8(n) | Reg::Xmm(n) => n < 16,
        Reg::R8H(n) => n < 4,
    }
}

/// gas's text for a memory operand, used in its "not a valid base/index expression" message.
fn mem_text(m: &Mem) -> String {
    let inner = match &m.base {
        Base::Reg(r) => {
            if m.disp == 0 {
                regs::name(*r)
            } else if m.disp < 0 {
                format!("{}-{}", regs::name(*r), m.disp.unsigned_abs())
            } else {
                format!("{}+{}", regs::name(*r), m.disp)
            }
        }
        Base::Rip(s) => {
            if m.disp == 0 {
                format!("rip+{s}")
            } else {
                format!("rip+{s}{:+}", m.disp)
            }
        }
        Base::Abs => format!("{}", m.disp),
    };
    let prefix = match m.size {
        None => "",
        Some(Size::Byte) => "BYTE PTR ",
        Some(Size::Word) => "WORD PTR ",
        Some(Size::Dword) => "DWORD PTR ",
        Some(Size::Qword) => "QWORD PTR ",
        Some(Size::Xmmword) => "XMMWORD PTR ",
    };
    format!("{prefix}[{inner}]")
}

/// A memory operand lowered to fadec form.
struct FeMem {
    op: FeOp,
    addr32: bool,
    rip: Option<(String, i64)>,
    rex: bool,
    size: Option<Sz>,
}

fn lower_mem(m: &Mem) -> Result<FeMem, String> {
    let size = m.size.map(Sz::of_size);
    let disp32 = |d: i64| -> Result<i32, String> {
        i32::try_from(d).map_err(|_| format!("{d:#x} shortened to {:#x}", d as u32))
    };
    match &m.base {
        Base::Reg(r @ (Reg::R64(n) | Reg::R32(n))) if *n < 16 => Ok(FeMem {
            op: mem(gp(*n), 0, FE_NOREG, disp32(m.disp)?),
            addr32: matches!(r, Reg::R32(_)),
            rip: None,
            rex: *n >= 8,
            size,
        }),
        Base::Reg(_) => Err(format!("`{}' is not a valid base/index expression", mem_text(m))),
        Base::Rip(sym) => Ok(FeMem {
            op: mem(FE_IP, 0, FE_NOREG, 0),
            addr32: false,
            rip: Some((sym.clone(), m.disp)),
            rex: false,
            size,
        }),
        Base::Abs => Ok(FeMem {
            op: mem(FE_NOREG, 0, FE_NOREG, disp32(m.disp)?),
            addr32: false,
            rip: None,
            rex: false,
            size,
        }),
    }
}

/// How an immediate is to be encoded.
struct Imm {
    /// Value handed to fadec (selects the imm8 vs. full-size form like gas does).
    fe: i64,
    /// Trailing immediate bytes to overwrite after encoding (`len`, value).
    patch: Option<(usize, u64)>,
}

fn fits_i8(v: i64) -> bool {
    (-128..=127).contains(&v)
}
fn fits_i32(v: i64) -> bool {
    i32::try_from(v).is_ok()
}
fn sext16(v: i64) -> i64 {
    v as u16 as i16 as i64
}
fn sext32(v: i64) -> i64 {
    v as u32 as i32 as i64
}

/// gas `optimize_imm` for an operand of size `sz`: `None` if gas would reject the value
/// ("operand type mismatch"), else the normalised value (from which the imm8 decision is made).
fn normalize_imm(sz: Sz, v: i64) -> Option<i64> {
    match sz {
        Sz::B => Some(v),
        Sz::W => {
            let mut x = v;
            if (0..=0xffff).contains(&x) {
                x = sext16(x);
            }
            if (0..=0xffff_ffff).contains(&x) {
                x = sext32(x);
            }
            Some(x)
        }
        Sz::D => {
            if (0..=0xffff_ffff).contains(&v) {
                Some(sext32(v))
            } else {
                Some(v)
            }
        }
        Sz::Q => fits_i32(v).then_some(v),
        Sz::X => None,
    }
}

/// Immediate for an instruction that has an imm8 (sign-extended) form when `imm8_form`.
fn imm_operand(sz: Sz, v: i64, imm8_form: bool) -> Option<Imm> {
    let x = normalize_imm(sz, v)?;
    Some(match sz {
        Sz::B => Imm { fe: v as i8 as i64, patch: None },
        _ if imm8_form && fits_i8(x) => Imm { fe: x, patch: None },
        Sz::W => Imm { fe: 0x1000, patch: Some((2, x as u64 & 0xffff)) },
        Sz::D => Imm { fe: 0x1000_0000, patch: Some((4, x as u64 & 0xffff_ffff)) },
        Sz::Q => Imm { fe: x, patch: None },
        Sz::X => return None,
    })
}

/// Mnemonic tables indexed by size (B, W, D, Q).
type M4 = [u64; 4];

struct AluForms {
    rr: M4,
    rm: M4,
    mr: M4,
    ri: M4,
    mi: M4,
}

fn alu_forms(op: Op) -> Option<AluForms> {
    macro_rules! f {
        ($p:ident) => {
            paste_forms!($p)
        };
    }
    macro_rules! paste_forms {
        (ADD) => { forms!(FE_ADD8rr FE_ADD16rr FE_ADD32rr FE_ADD64rr; FE_ADD8rm FE_ADD16rm FE_ADD32rm FE_ADD64rm; FE_ADD8mr FE_ADD16mr FE_ADD32mr FE_ADD64mr; FE_ADD8ri FE_ADD16ri FE_ADD32ri FE_ADD64ri; FE_ADD8mi FE_ADD16mi FE_ADD32mi FE_ADD64mi) };
        (SUB) => { forms!(FE_SUB8rr FE_SUB16rr FE_SUB32rr FE_SUB64rr; FE_SUB8rm FE_SUB16rm FE_SUB32rm FE_SUB64rm; FE_SUB8mr FE_SUB16mr FE_SUB32mr FE_SUB64mr; FE_SUB8ri FE_SUB16ri FE_SUB32ri FE_SUB64ri; FE_SUB8mi FE_SUB16mi FE_SUB32mi FE_SUB64mi) };
        (AND) => { forms!(FE_AND8rr FE_AND16rr FE_AND32rr FE_AND64rr; FE_AND8rm FE_AND16rm FE_AND32rm FE_AND64rm; FE_AND8mr FE_AND16mr FE_AND32mr FE_AND64mr; FE_AND8ri FE_AND16ri FE_AND32ri FE_AND64ri; FE_AND8mi FE_AND16mi FE_AND32mi FE_AND64mi) };
        (XOR) => { forms!(FE_XOR8rr FE_XOR16rr FE_XOR32rr FE_XOR64rr; FE_XOR8rm FE_XOR16rm FE_XOR32rm FE_XOR64rm; FE_XOR8mr FE_XOR16mr FE_XOR32mr FE_XOR64mr; FE_XOR8ri FE_XOR16ri FE_XOR32ri FE_XOR64ri; FE_XOR8mi FE_XOR16mi FE_XOR32mi FE_XOR64mi) };
        (CMP) => { forms!(FE_CMP8rr FE_CMP16rr FE_CMP32rr FE_CMP64rr; FE_CMP8rm FE_CMP16rm FE_CMP32rm FE_CMP64rm; FE_CMP8mr FE_CMP16mr FE_CMP32mr FE_CMP64mr; FE_CMP8ri FE_CMP16ri FE_CMP32ri FE_CMP64ri; FE_CMP8mi FE_CMP16mi FE_CMP32mi FE_CMP64mi) };
        (MOV) => { forms!(FE_MOV8rr FE_MOV16rr FE_MOV32rr FE_MOV64rr; FE_MOV8rm FE_MOV16rm FE_MOV32rm FE_MOV64rm; FE_MOV8mr FE_MOV16mr FE_MOV32mr FE_MOV64mr; FE_MOV8ri FE_MOV16ri FE_MOV32ri FE_MOV64ri; FE_MOV8mi FE_MOV16mi FE_MOV32mi FE_MOV64mi) };
        // test has no r,m form; gas encodes `test r, m` as `test m, r` (85 /r).
        (TEST) => { forms!(FE_TEST8rr FE_TEST16rr FE_TEST32rr FE_TEST64rr; FE_TEST8mr FE_TEST16mr FE_TEST32mr FE_TEST64mr; FE_TEST8mr FE_TEST16mr FE_TEST32mr FE_TEST64mr; FE_TEST8ri FE_TEST16ri FE_TEST32ri FE_TEST64ri; FE_TEST8mi FE_TEST16mi FE_TEST32mi FE_TEST64mi) };
    }
    macro_rules! forms {
        ($($a:ident)*; $($b:ident)*; $($c:ident)*; $($d:ident)*; $($e:ident)*) => {
            AluForms { rr: [$($a),*], rm: [$($b),*], mr: [$($c),*], ri: [$($d),*], mi: [$($e),*] }
        };
    }
    Some(match op {
        Op::Add => f!(ADD),
        Op::Sub => f!(SUB),
        Op::And => f!(AND),
        Op::Xor => f!(XOR),
        Op::Cmp => f!(CMP),
        Op::Mov => f!(MOV),
        Op::Test => f!(TEST),
        _ => return None,
    })
}

/// Accumulates the pieces of one fadec call.
struct Insn {
    mnem: u64,
    ops: Vec<FeOp>,
    imm_patch: Option<(usize, u64)>,
    rip: Option<(String, i64)>,
    mem_idx: Option<usize>,
}

impl Insn {
    fn new(mnem: u64) -> Insn {
        Insn { mnem, ops: Vec::new(), imm_patch: None, rip: None, mem_idx: None }
    }
    fn reg(mut self, r: Reg) -> Insn {
        self.ops.push(reg_op(r));
        self
    }
    fn mem(mut self, m: &FeMem) -> Insn {
        self.mem_idx = Some(self.ops.len());
        self.ops.push(m.op);
        if m.addr32 {
            self.mnem |= FE_ADDR32;
        }
        self.rip.clone_from(&m.rip);
        self
    }
    fn imm(mut self, i: Imm) -> Insn {
        self.ops.push(i.fe);
        self.imm_patch = i.patch;
        self
    }
    fn raw(mut self, v: i64) -> Insn {
        self.ops.push(v);
        self
    }
    fn encode(self, name: &str) -> Result<Encoded, String> {
        let err = || format!("cannot encode `{name}' with these operands");
        let mut bytes = fadec::encode(self.mnem, &self.ops).map_err(|_| err())?;
        if let Some((len, v)) = self.imm_patch {
            let n = bytes.len();
            bytes[n - len..].copy_from_slice(&v.to_le_bytes()[..len]);
        }
        let rip = match self.rip {
            None => None,
            Some((sym, disp)) => {
                // Locate the disp32 field: re-encode with a different RIP offset and see which
                // bytes change.
                let mut ops = self.ops.clone();
                let mi = self.mem_idx.expect("rip operand without memory operand");
                ops[mi] = mem(FE_IP, 0, FE_NOREG, 0x1234_5678);
                let other = fadec::encode(self.mnem, &ops).map_err(|_| err())?;
                let field = (0..bytes.len()).find(|&i| bytes[i] != other[i]).expect("disp field");
                bytes[field..field + 4].fill(0);
                Some((field, sym, disp))
            }
        };
        Ok(Encoded { bytes, rip })
    }
}

struct Ctx<'a> {
    name: &'a str,
}

impl Ctx<'_> {
    /// A form gas accepts but the Emitter doesn't implement (the translator never produces it).
    fn unsupported(&self, n: usize) -> String {
        let s = if n == 1 { "" } else { "s" };
        format!("`{}' with {n} operand{s} is not supported by the Emitter", self.name)
    }
    fn mismatch(&self) -> String {
        format!("operand type mismatch for `{}'", self.name)
    }
    fn reg_mismatch(&self) -> String {
        format!("register type mismatch for `{}'", self.name)
    }
    fn size_mismatch(&self) -> String {
        format!("operand size mismatch for `{}'", self.name)
    }
    fn ambiguous(&self) -> String {
        format!("ambiguous operand size for `{}'", self.name)
    }
}

fn gpr(r: Reg) -> bool {
    !matches!(r, Reg::Xmm(_))
}

/// Encode a non-branch instruction.
pub(super) fn encode(op: Op, ops: &[Operand]) -> Result<Encoded, String> {
    let name = mnemonic(op);
    let cx = Ctx { name };

    // Operand counts no gas template accepts.
    let arity_ok = match op {
        Op::Cqo | Op::Hlt | Op::Int3 | Op::Syscall => ops.is_empty(),
        Op::Ret | Op::Nop => ops.len() <= 1,
        Op::Push | Op::Pop | Op::Rdrand => ops.len() == 1,
        Op::Mov | Op::Movups | Op::Lea | Op::Add | Op::Sub | Op::And | Op::Xor | Op::Cmp | Op::Test => ops.len() == 2,
        // (shifts, not/inc/dec/idiv and imul also have one-, two- or three-operand forms)
        Op::Not | Op::Inc | Op::Dec | Op::Idiv | Op::Shl | Op::Shr | Op::Imul => (1..=3).contains(&ops.len()),
    };
    if !arity_ok {
        return Err(format!("number of operands mismatch for `{name}'"));
    }

    // Validate registers and lower memory operands first (gas reports bad base registers while
    // parsing the operand, before template matching).
    let mut mems: Vec<FeMem> = Vec::new();
    let mut size_mod: Option<Size> = None;
    // Address size of the first memory operand with a base register (`Some(true)` = 32-bit).
    let mut addr_size: Option<bool> = None;
    for o in ops {
        match o {
            Operand::Reg(r) if !valid_reg(*r) => {
                return Err(format!("bad register {r:?} for `{name}'"));
            }
            Operand::Mem(m) => {
                // `BYTE PTR` .. `QWORD PTR` set the mnemonic suffix where a template allows one
                // (not for lea/movups; `XMMWORD PTR` never does); two different suffixes conflict.
                // A byte suffix after another one is dropped for imul (its two-operand
                // template has no byte form), so that pair ends as a type mismatch.
                let suffix = m.size.filter(|s| *s != Size::Xmmword && !matches!(op, Op::Lea | Op::Movups));
                if let Some(sz) = suffix {
                    if !(op == Op::Imul && sz == Size::Byte && size_mod.is_some()) {
                        if size_mod.is_some_and(|p| p != sz) {
                            return Err("conflicting operand size modifiers".into());
                        }
                        size_mod = Some(sz);
                    }
                }
                let fm = lower_mem(m)?;
                let this = match m.base {
                    Base::Abs => None,
                    _ => Some(fm.addr32),
                };
                if let (Some(a), Some(b)) = (addr_size, this) {
                    if a != b {
                        return Err(format!("`{}' is not a valid base/index expression", mem_text(m)));
                    }
                }
                addr_size = addr_size.or(this);
                mems.push(fm);
            }
            _ => {}
        }
    }
    if mems.len() > 1 {
        // Two memory operands never match a template.
        let sized = |m: &FeMem| m.size.is_some_and(|s| s != Sz::X);
        let x = |m: &FeMem| m.size == Some(Sz::X);
        return Err(match op {
            Op::Movups if mems.iter().all(sized) => cx.size_mismatch(),
            Op::Shl | Op::Shr if x(&mems[0]) => cx.size_mismatch(),
            Op::Not | Op::Inc | Op::Dec | Op::Idiv if !x(&mems[0]) && x(&mems[1]) => cx.size_mismatch(),
            _ => cx.mismatch(),
        });
    }
    let fm = mems.into_iter().next();

    let enc = select(op, ops, fm.as_ref(), &cx)?;

    // A legacy high-byte register can't be combined with anything needing a REX prefix.
    let high = ops.iter().find_map(|o| match o {
        Operand::Reg(r @ Reg::R8H(_)) => Some(*r),
        _ => None,
    });
    if let Some(h) = high {
        let rex = ops.iter().any(|o| match o {
            Operand::Reg(r) => reg_needs_rex(*r),
            _ => false,
        }) || fm.as_ref().is_some_and(|m| m.rex);
        if rex {
            return Err(format!(
                "can't encode register '{}' in an instruction requiring REX/REX2 prefix",
                regs::name(h)
            ));
        }
    }

    enc.encode(name)
}

/// Pick the fadec mnemonic + operands like gas picks its template.
fn select(op: Op, ops: &[Operand], fm: Option<&FeMem>, cx: &Ctx) -> Result<Insn, String> {
    use Operand as O;
    let none_ops = || -> Result<(), String> {
        if ops.is_empty() {
            Ok(())
        } else {
            Err(cx.mismatch())
        }
    };
    match op {
        Op::Cqo => none_ops().map(|_| Insn::new(FE_CQO)),
        // `ret imm16` and `nop r/m` exist but are never produced by the translator.
        Op::Ret | Op::Nop if matches!(ops, [O::Imm(_)]) == (op == Op::Ret) && !ops.is_empty() => {
            Err(cx.unsupported(1))
        }
        Op::Ret => none_ops().map(|_| Insn::new(FE_RET)),
        Op::Nop => none_ops().map(|_| Insn::new(FE_NOP)),
        Op::Hlt => none_ops().map(|_| Insn::new(FE_HLT)),
        Op::Int3 => none_ops().map(|_| Insn::new(FE_INT3)),
        Op::Syscall => none_ops().map(|_| Insn::new(FE_SYSCALL)),

        Op::Mov | Op::Add | Op::Sub | Op::And | Op::Xor | Op::Cmp | Op::Test => {
            let f = alu_forms(op).unwrap();
            let [a, b] = ops else { return Err(cx.mismatch()) };
            match (a, b) {
                (O::Reg(x), O::Reg(y)) => {
                    if !gpr(*x) || !gpr(*y) {
                        return Err(cx.mismatch());
                    }
                    let (sx, sy) = (reg_sz(*x), reg_sz(*y));
                    if sx != sy {
                        return Err(if op == Op::Mov { cx.mismatch() } else { cx.reg_mismatch() });
                    }
                    Ok(Insn::new(f.rr[sx.idx()]).reg(*x).reg(*y))
                }
                (O::Reg(x), O::Mem(_)) | (O::Mem(_), O::Reg(x)) => {
                    let m = fm.unwrap();
                    if !gpr(*x) {
                        return Err(cx.mismatch());
                    }
                    let sx = reg_sz(*x);
                    if let Some(s) = m.size.filter(|s| *s != sx) {
                        let reg_first = matches!(a, O::Reg(_));
                        let reg = reg_first && op != Op::Mov && s != Sz::X;
                        return Err(if reg { cx.reg_mismatch() } else { cx.mismatch() });
                    }
                    let i = if matches!(a, O::Reg(_)) {
                        if op == Op::Test {
                            Insn::new(f.mr[sx.idx()]).mem(m).reg(*x)
                        } else {
                            Insn::new(f.rm[sx.idx()]).reg(*x).mem(m)
                        }
                    } else {
                        Insn::new(f.mr[sx.idx()]).mem(m).reg(*x)
                    };
                    Ok(i)
                }
                (O::Reg(x), O::Imm(v)) => {
                    if !gpr(*x) {
                        return Err(cx.mismatch());
                    }
                    let sz = reg_sz(*x);
                    if op == Op::Mov && sz == Sz::Q {
                        // gas: imm32s → c7 /0, otherwise movabs (b8+r io); fadec agrees.
                        return Ok(Insn::new(FE_MOV64ri).reg(*x).raw(*v));
                    }
                    let imm8 = !matches!(op, Op::Mov | Op::Test);
                    let i = imm_operand(sz, *v, imm8).ok_or_else(|| cx.mismatch())?;
                    Ok(Insn::new(f.ri[sz.idx()]).reg(*x).imm(i))
                }
                (O::Mem(_), O::Imm(v)) => {
                    let m = fm.unwrap();
                    let sz = m.size.ok_or_else(|| cx.ambiguous())?;
                    if sz == Sz::X {
                        return Err(cx.mismatch());
                    }
                    let imm8 = !matches!(op, Op::Mov | Op::Test);
                    let i = imm_operand(sz, *v, imm8).ok_or_else(|| cx.mismatch())?;
                    Ok(Insn::new(f.mi[sz.idx()]).mem(m).imm(i))
                }
                _ => Err(cx.mismatch()),
            }
        }

        Op::Not | Op::Inc | Op::Dec | Op::Idiv => {
            let t: (M4, M4) = match op {
                Op::Not => ([FE_NOT8r, FE_NOT16r, FE_NOT32r, FE_NOT64r], [FE_NOT8m, FE_NOT16m, FE_NOT32m, FE_NOT64m]),
                Op::Inc => ([FE_INC8r, FE_INC16r, FE_INC32r, FE_INC64r], [FE_INC8m, FE_INC16m, FE_INC32m, FE_INC64m]),
                Op::Dec => ([FE_DEC8r, FE_DEC16r, FE_DEC32r, FE_DEC64r], [FE_DEC8m, FE_DEC16m, FE_DEC32m, FE_DEC64m]),
                _ => ([FE_IDIV8r, FE_IDIV16r, FE_IDIV32r, FE_IDIV64r], [FE_IDIV8m, FE_IDIV16m, FE_IDIV32m, FE_IDIV64m]),
            };
            match ops {
                [O::Reg(r)] if gpr(*r) => Ok(Insn::new(t.0[reg_sz(*r).idx()]).reg(*r)),
                [O::Mem(_)] => {
                    let m = fm.unwrap();
                    let sz = m.size.ok_or_else(|| cx.ambiguous())?;
                    if sz == Sz::X {
                        return Err(cx.size_mismatch());
                    }
                    Ok(Insn::new(t.1[sz.idx()]).mem(m))
                }
                // `reg, XMMWORD PTR [..]` fails size matching against the NDD / accumulator forms.
                [O::Reg(_), O::Mem(_)] if fm.unwrap().size == Some(Sz::X) => Err(cx.size_mismatch()),
                // Two- and three-operand forms (APX NDD `not`/`inc`/`dec`, `idiv al, bl`) exist in
                // gas but are never produced by the translator.
                [O::Reg(r), O::Reg(_) | O::Mem(_), ..] if gpr(*r) => Err(cx.unsupported(ops.len())),
                _ => Err(cx.mismatch()),
            }
        }

        Op::Shl | Op::Shr => {
            let (ri, mi, rc, mc): (M4, M4, M4, M4) = if op == Op::Shl {
                (
                    [FE_SHL8ri, FE_SHL16ri, FE_SHL32ri, FE_SHL64ri],
                    [FE_SHL8mi, FE_SHL16mi, FE_SHL32mi, FE_SHL64mi],
                    [FE_SHL8rr, FE_SHL16rr, FE_SHL32rr, FE_SHL64rr],
                    [FE_SHL8mr, FE_SHL16mr, FE_SHL32mr, FE_SHL64mr],
                )
            } else {
                (
                    [FE_SHR8ri, FE_SHR16ri, FE_SHR32ri, FE_SHR64ri],
                    [FE_SHR8mi, FE_SHR16mi, FE_SHR32mi, FE_SHR64mi],
                    [FE_SHR8rr, FE_SHR16rr, FE_SHR32rr, FE_SHR64rr],
                    [FE_SHR8mr, FE_SHR16mr, FE_SHR32mr, FE_SHR64mr],
                )
            };
            if matches!(ops[0], O::Mem(_)) && fm.unwrap().size == Some(Sz::X) {
                return Err(cx.size_mismatch());
            }
            let [a, count] = ops else {
                return Err(match ops {
                    [O::Mem(_)] if fm.unwrap().size.is_none() => cx.ambiguous(),
                    // `shl rax` (shift by one) is valid gas but never produced by the translator.
                    [O::Mem(_)] => cx.unsupported(1),
                    [O::Reg(r)] if gpr(*r) => cx.unsupported(1),
                    _ => cx.mismatch(),
                });
            };
            // Only `cl` or an immediate count. Without a size, a count that fits no imm8 template
            // is a type mismatch; otherwise the size is ambiguous.
            let count = match count {
                O::Reg(Reg::R8(1)) => None,
                O::Imm(v) => Some(*v),
                _ => return Err(cx.mismatch()),
            };
            let sz = match a {
                O::Reg(r) if gpr(*r) => reg_sz(*r),
                O::Mem(_) => match fm.unwrap().size {
                    Some(s) => s,
                    None if count.is_none_or(|v| (0..=255).contains(&v)) => return Err(cx.ambiguous()),
                    None => return Err(cx.mismatch()),
                },
                _ => return Err(cx.mismatch()),
            };
            if sz == Sz::X {
                return Err(cx.mismatch());
            }
            let x = sz.idx();
            let insn = |r: M4, m: M4| match a {
                O::Reg(reg) => Insn::new(r[x]).reg(*reg),
                _ => Insn::new(m[x]).mem(fm.unwrap()),
            };
            match count {
                None => Ok(insn(rc, mc).reg(Reg::R8(1))),
                // Byte operations truncate any count; the others need 0..=255 as written.
                // fadec picks the D0/D1 form for a count of 1, like gas.
                Some(v) if sz == Sz::B || (0..=255).contains(&v) => Ok(insn(ri, mi).raw(v as u8 as i8 as i64)),
                Some(_) => Err(cx.mismatch()),
            }
        }

        Op::Imul => match ops {
            [O::Reg(r)] if gpr(*r) => {
                let t = [FE_IMUL8r, FE_IMUL16r, FE_IMUL32r, FE_IMUL64r];
                Ok(Insn::new(t[reg_sz(*r).idx()]).reg(*r))
            }
            [O::Mem(_)] => {
                let m = fm.unwrap();
                let sz = m.size.ok_or_else(|| cx.ambiguous())?;
                let t = [FE_IMUL8m, FE_IMUL16m, FE_IMUL32m, FE_IMUL64m];
                if sz == Sz::X {
                    return Err(cx.size_mismatch());
                }
                Ok(Insn::new(t[sz.idx()]).mem(m))
            }
            // Two-operand imul has no byte form: gas rejects it during size matching.
            [O::Reg(x), _] if gpr(*x) && reg_sz(*x) == Sz::B => Err(cx.size_mismatch()),
            [O::Reg(x), O::Reg(y)] => {
                if !gpr(*x) || !gpr(*y) {
                    return Err(cx.mismatch());
                }
                let (sx, sy) = (reg_sz(*x), reg_sz(*y));
                if sy == Sz::B {
                    return Err(cx.mismatch());
                }
                if sx != sy {
                    return Err(cx.reg_mismatch());
                }
                let t = [0, FE_IMUL16rr, FE_IMUL32rr, FE_IMUL64rr];
                Ok(Insn::new(t[sx.idx()]).reg(*x).reg(*y))
            }
            [O::Reg(x), O::Mem(_)] => {
                let m = fm.unwrap();
                if !gpr(*x) {
                    return Err(cx.mismatch());
                }
                let sx = reg_sz(*x);
                if let Some(s) = m.size.filter(|s| *s != sx) {
                    return Err(if matches!(s, Sz::B | Sz::X) { cx.mismatch() } else { cx.reg_mismatch() });
                }
                let t = [0, FE_IMUL16rm, FE_IMUL32rm, FE_IMUL64rm];
                Ok(Insn::new(t[sx.idx()]).reg(*x).mem(m))
            }
            [O::Reg(x), O::Imm(v)] => {
                if !gpr(*x) {
                    return Err(cx.mismatch());
                }
                let sx = reg_sz(*x);
                let i = imm_operand(sx, *v, true).ok_or_else(|| cx.mismatch())?;
                let t = [0, FE_IMUL16rri, FE_IMUL32rri, FE_IMUL64rri];
                Ok(Insn::new(t[sx.idx()]).reg(*x).reg(*x).imm(i))
            }
            // `imul r, r/m, imm` and the APX NDD form are never produced by the translator.
            [O::Reg(x), O::Reg(_) | O::Mem(_), _] if gpr(*x) => Err(cx.unsupported(3)),
            _ => Err(cx.mismatch()),
        },

        Op::Push | Op::Pop => {
            let push = op == Op::Push;
            match ops {
                [O::Reg(r)] => match r {
                    Reg::R64(_) => Ok(Insn::new(if push { FE_PUSHr } else { FE_POPr }).reg(*r)),
                    Reg::R16(_) => Ok(Insn::new(if push { FE_PUSH16r } else { FE_POP16r }).reg(*r)),
                    _ => Err(cx.mismatch()),
                },
                [O::Mem(_)] => {
                    let m = fm.unwrap();
                    let mn = match m.size {
                        None | Some(Sz::Q) => {
                            if push {
                                FE_PUSHm
                            } else {
                                FE_POPm
                            }
                        }
                        Some(Sz::W) => {
                            if push {
                                FE_PUSH16m
                            } else {
                                FE_POP16m
                            }
                        }
                        _ => return Err(cx.mismatch()),
                    };
                    Ok(Insn::new(mn).mem(m))
                }
                [O::Imm(v)] if push => {
                    if !fits_i32(*v) {
                        return Err(cx.mismatch());
                    }
                    Ok(Insn::new(FE_PUSHi).raw(*v))
                }
                _ => Err(cx.mismatch()),
            }
        }

        Op::Lea => match ops {
            [O::Reg(x), _] if gpr(*x) && reg_sz(*x) == Sz::B => Err(cx.size_mismatch()),
            [O::Reg(x), O::Mem(_)] if gpr(*x) => {
                let t = [0, FE_LEA16rm, FE_LEA32rm, FE_LEA64rm];
                let sx = reg_sz(*x);
                if sx == Sz::B {
                    return Err(cx.size_mismatch());
                }
                Ok(Insn::new(t[sx.idx()]).reg(*x).mem(fm.unwrap()))
            }
            _ => Err(cx.mismatch()),
        },

        Op::Rdrand => match ops {
            [O::Reg(r)] if gpr(*r) => {
                let mn = match reg_sz(*r) {
                    Sz::W => FE_RDRAND16r,
                    Sz::D => FE_RDRAND32r,
                    Sz::Q => FE_RDRAND64r,
                    _ => return Err(cx.size_mismatch()),
                };
                Ok(Insn::new(mn).reg(*r))
            }
            _ => Err(cx.mismatch()),
        },

        Op::Movups => {
            let okm = |m: &FeMem| m.size.is_none() || m.size == Some(Sz::X);
            match ops {
                [O::Reg(x @ Reg::Xmm(_)), O::Reg(y @ Reg::Xmm(_))] => {
                    Ok(Insn::new(FE_SSE_MOVUPSrr).reg(*x).reg(*y))
                }
                [O::Reg(x @ Reg::Xmm(_)), O::Mem(_)] if okm(fm.unwrap()) => {
                    Ok(Insn::new(FE_SSE_MOVUPSrm).reg(*x).mem(fm.unwrap()))
                }
                [O::Mem(_), O::Reg(x @ Reg::Xmm(_))] if okm(fm.unwrap()) => {
                    Ok(Insn::new(FE_SSE_MOVUPSmr).mem(fm.unwrap()).reg(*x))
                }
                _ => Err(cx.mismatch()),
            }
        }
    }
}
