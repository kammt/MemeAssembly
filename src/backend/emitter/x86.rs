//! A small x86-64 encoder for the instruction forms the translator emits. The encodings are the
//! ones GNU as picks for the equivalent Intel-syntax line: the store form (`01 /r`) for
//! register-to-register operations, an imm8 when the value fits sign-extended, the short
//! accumulator forms (`add eax, imm32` = `05 id`), `mov r64, imm` as `c7 /0 id` when the value
//! fits a sign-extended imm32 (else `movabs`), `d0`/`d1` for shifts by one, a SIB byte for
//! `rsp`/`r12` bases, a zero disp8 for `rbp`/`r13` bases, and `0x67` for 32-bit bases.
//!
//! Operand combinations the translator never produces are rejected with one internal error.

use super::{Base, Cond, Mem, Op, Operand, Reg, Size};

/// An encoded instruction plus the location of its RIP-relative displacement, if any.
pub(crate) struct Encoded {
    pub bytes: Vec<u8>,
    /// `(offset of the disp32 field, symbol, displacement)`: the field (left zero) must hold
    /// `symbol + displacement - end_of_instruction`.
    pub rip: Option<(usize, String, i64)>,
}

/// Operand size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sz {
    B,
    W,
    D,
    Q,
}

/// `None` for the XMM registers, which have no general-purpose operand size.
fn reg_size(r: Reg) -> Option<Sz> {
    match r {
        Reg::R64(_) => Some(Sz::Q),
        Reg::R32(_) => Some(Sz::D),
        Reg::R16(_) => Some(Sz::W),
        Reg::R8(_) | Reg::R8H(_) => Some(Sz::B),
        Reg::Xmm(_) => None,
    }
}

fn mem_size(m: &Mem) -> Option<Sz> {
    match m.size? {
        Size::Byte => Some(Sz::B),
        Size::Word => Some(Sz::W),
        Size::Dword => Some(Sz::D),
        Size::Qword => Some(Sz::Q),
    }
}

/// Register number as encoded (0..=15; `ah`..`bh` are 4..=7 without REX).
fn reg_num(r: Reg) -> u8 {
    match r {
        Reg::R64(n) | Reg::R32(n) | Reg::R16(n) | Reg::R8(n) | Reg::Xmm(n) => n,
        Reg::R8H(n) => 4 + n,
    }
}

/// `spl`/`bpl`/`sil`/`dil` are only addressable with a REX prefix.
fn needs_empty_rex(r: Reg) -> bool {
    matches!(r, Reg::R8(4..=7))
}

fn fits_i8(v: i64) -> bool {
    (-128..=127).contains(&v)
}

fn fits_i32(v: i64) -> bool {
    i32::try_from(v).is_ok()
}

/// The `reg` field of the ModRM byte: a register or an opcode extension (`/digit`).
#[derive(Clone, Copy)]
enum Field {
    Reg(Reg),
    Digit(u8),
}

/// The `r/m` operand.
#[derive(Clone, Copy)]
enum Rm<'a> {
    Reg(Reg),
    Mem(&'a Mem),
}

/// An immediate: absent, a sign-extended imm8 or the full operand-size immediate.
#[derive(Clone, Copy)]
enum Imm {
    None,
    I8(i64),
    Full(i64, usize),
}

impl Imm {
    /// gas's choice for an immediate `v` of an operand of size `sz` (with `imm8_form` when the
    /// instruction has a sign-extended imm8 variant). Byte, word and dword immediates outside
    /// the operand's range are truncated; a qword immediate must be a sign-extended imm32.
    fn new(sz: Sz, v: i64, imm8_form: bool) -> Option<Imm> {
        let x = match sz {
            Sz::B => return Some(Imm::Full(v, 1)),
            Sz::W if (0..=0xffff).contains(&v) => v as i16 as i64,
            Sz::D | Sz::W if (0..=0xffff_ffff).contains(&v) => v as i32 as i64,
            Sz::Q if !fits_i32(v) => return None,
            _ => v,
        };
        Some(if imm8_form && fits_i8(x) {
            Imm::I8(x)
        } else {
            Imm::Full(x, if sz == Sz::W { 2 } else { 4 })
        })
    }

    fn put(self, out: &mut Vec<u8>) {
        match self {
            Imm::None => {}
            Imm::I8(v) => out.push(v as u8),
            Imm::Full(v, n) => out.extend_from_slice(&v.to_le_bytes()[..n]),
        }
    }
}

/// The result of an encoding helper; `rip` as in [`Encoded`].
struct Insn {
    bytes: Vec<u8>,
    rip: Option<(usize, String, i64)>,
}

/// Legacy prefixes, REX and opcode for an instruction of operand size `sz` (`None`: no
/// 0x66/REX.W) whose registers are `regs` (with the REX bit each one sets).
fn prefix(out: &mut Vec<u8>, sz: Option<Sz>, addr32: bool, rex_rb: u8, regs: &[Reg]) -> Option<()> {
    if addr32 {
        out.push(0x67);
    }
    if sz == Some(Sz::W) {
        out.push(0x66);
    }
    let rex = rex_rb | if sz == Some(Sz::Q) { 8 } else { 0 };
    if rex != 0 || regs.iter().any(|r| needs_empty_rex(*r)) {
        // The legacy high-byte registers can't be encoded together with a REX prefix.
        if regs.iter().any(|r| matches!(r, Reg::R8H(_))) {
            return None;
        }
        out.push(0x40 | rex);
    }
    Some(())
}

/// `[prefixes] opcode ModRM [SIB] [disp] [imm]`
fn modrm(sz: Option<Sz>, opcode: &[u8], field: Field, rm: Rm, imm: Imm) -> Option<Insn> {
    let (reg, mut regs) = match field {
        Field::Reg(r) => (reg_num(r), vec![r]),
        Field::Digit(d) => (d, Vec::new()),
    };
    let mut rex = (reg >> 3) << 2;
    let mut out = Vec::with_capacity(16);
    let mut tail = Vec::with_capacity(9);
    let mut rip = None;
    let modrm_byte;
    let mut addr32 = false;
    match rm {
        Rm::Reg(r) => {
            regs.push(r);
            let n = reg_num(r);
            rex |= n >> 3;
            modrm_byte = 0xc0 | (reg & 7) << 3 | (n & 7);
        }
        Rm::Mem(m) => match &m.base {
            Base::Reg(b @ (Reg::R64(n) | Reg::R32(n))) => {
                let (n, disp) = (*n, i32::try_from(m.disp).ok()?);
                addr32 = matches!(b, Reg::R32(_));
                rex |= n >> 3;
                let md = if disp == 0 && n & 7 != 5 {
                    0x00
                } else if fits_i8(disp as i64) {
                    0x40
                } else {
                    0x80
                };
                modrm_byte = md | (reg & 7) << 3 | (n & 7);
                if n & 7 == 4 {
                    tail.push(0x24);
                }
                match md {
                    0x40 => tail.push(disp as u8),
                    0x80 => tail.extend_from_slice(&disp.to_le_bytes()),
                    _ => {}
                }
            }
            Base::Reg(_) => return None,
            Base::Rip(sym) => {
                modrm_byte = 0x05 | (reg & 7) << 3;
                rip = Some((sym.clone(), m.disp));
                tail.extend_from_slice(&[0; 4]);
            }
            Base::Abs => {
                modrm_byte = 0x04 | (reg & 7) << 3;
                tail.push(0x25);
                tail.extend_from_slice(&i32::try_from(m.disp).ok()?.to_le_bytes());
            }
        },
    }
    prefix(&mut out, sz, addr32, rex, &regs)?;
    out.extend_from_slice(opcode);
    out.push(modrm_byte);
    out.extend_from_slice(&tail);
    // (the RIP displacement is the last 4 bytes of `tail`)
    let field_at = out.len().wrapping_sub(4);
    imm.put(&mut out);
    Some(Insn { bytes: out, rip: rip.map(|(s, d)| (field_at, s, d)) })
}

/// `[prefixes] opcode+reg [imm]` (`push r64`, `mov r, imm`).
fn plus_reg(sz: Option<Sz>, opcode: u8, r: Reg, imm: Imm) -> Option<Insn> {
    let mut out = Vec::with_capacity(10);
    let n = reg_num(r);
    prefix(&mut out, sz, false, n >> 3, &[r])?;
    out.push(opcode + (n & 7));
    imm.put(&mut out);
    Some(Insn { bytes: out, rip: None })
}

/// An instruction without operands.
fn fixed(bytes: &[u8]) -> Option<Insn> {
    Some(Insn { bytes: bytes.to_vec(), rip: None })
}

/// Picks the opcode of a byte-size (`b`) or larger (`b + 1`) operation.
fn op8(sz: Sz, b: u8) -> u8 {
    if sz == Sz::B {
        b
    } else {
        b + 1
    }
}

/// Encode a non-branch instruction (Intel operand order, destination first).
pub(crate) fn encode(op: Op, ops: &[Operand]) -> Result<Encoded, String> {
    select(op, ops)
        .map(|i| Encoded { bytes: i.bytes, rip: i.rip })
        .ok_or_else(|| {
            let op = format!("{op:?}").to_ascii_lowercase();
            format!("internal error: unsupported operand combination for `{op}'")
        })
}

/// The encoding of `op` with `ops`, or `None` if the combination isn't supported.
fn select(op: Op, ops: &[Operand]) -> Option<Insn> {
    use Operand as O;
    match (op, ops) {
        (Op::Cqo, []) => fixed(&[0x48, 0x99]),
        (Op::Ret, []) => fixed(&[0xc3]),
        (Op::Nop, []) => fixed(&[0x90]),
        (Op::Hlt, []) => fixed(&[0xf4]),
        (Op::Int3, []) => fixed(&[0xcc]),
        (Op::Syscall, []) => fixed(&[0x0f, 0x05]),

        (Op::Mov | Op::Add | Op::Sub | Op::And | Op::Xor | Op::Cmp | Op::Test, [a, b]) => {
            // (r/m ← r opcode, /digit of the immediate group)
            let (store, digit) = match op {
                Op::Add => (0x00, 0),
                Op::And => (0x20, 4),
                Op::Sub => (0x28, 5),
                Op::Xor => (0x30, 6),
                Op::Cmp => (0x38, 7),
                Op::Mov => (0x88, 0),
                _ => (0x84, 0), // test
            };
            match (a, b) {
                (O::Reg(x), O::Reg(y)) => {
                    let sz = reg_size(*x).filter(|s| reg_size(*y) == Some(*s))?;
                    modrm(Some(sz), &[op8(sz, store)], Field::Reg(*y), Rm::Reg(*x), Imm::None)
                }
                (O::Reg(x), O::Mem(m)) | (O::Mem(m), O::Reg(x)) => {
                    let sz = reg_size(*x).filter(|s| m.size.is_none() || mem_size(m) == Some(*s))?;
                    // Load form `03 /r` for `reg, mem` (test has only the `85 /r` form).
                    let load = matches!(a, O::Reg(_)) && op != Op::Test;
                    let opc = op8(sz, store) + if load { 2 } else { 0 };
                    modrm(Some(sz), &[opc], Field::Reg(*x), Rm::Mem(m), Imm::None)
                }
                (O::Reg(x), O::Imm(v)) => {
                    let sz = reg_size(*x)?;
                    if op == Op::Mov {
                        return match sz {
                            Sz::Q if fits_i32(*v) => {
                                modrm(Some(sz), &[0xc7], Field::Digit(0), Rm::Reg(*x), Imm::Full(*v, 4))
                            }
                            Sz::Q => plus_reg(Some(sz), 0xb8, *x, Imm::Full(*v, 8)),
                            Sz::B => plus_reg(Some(sz), 0xb0, *x, Imm::new(sz, *v, false)?),
                            _ => plus_reg(Some(sz), 0xb8, *x, Imm::new(sz, *v, false)?),
                        };
                    }
                    let imm = Imm::new(sz, *v, op != Op::Test)?;
                    match imm {
                        Imm::I8(_) => modrm(Some(sz), &[0x83], Field::Digit(digit), Rm::Reg(*x), imm),
                        // Accumulator forms: `04/05 + store` (`a8/a9` for test).
                        _ if matches!(x, Reg::R64(0) | Reg::R32(0) | Reg::R16(0) | Reg::R8(0)) => {
                            let mut out = Vec::new();
                            prefix(&mut out, Some(sz), false, 0, &[])?;
                            let acc = if op == Op::Test { 0xa8 } else { store + 4 };
                            out.push(op8(sz, acc));
                            imm.put(&mut out);
                            Some(Insn { bytes: out, rip: None })
                        }
                        _ => {
                            let group = if op == Op::Test { 0xf6 } else { 0x80 };
                            modrm(Some(sz), &[op8(sz, group)], Field::Digit(digit), Rm::Reg(*x), imm)
                        }
                    }
                }
                (O::Mem(m), O::Imm(v)) => {
                    let sz = mem_size(m)?;
                    let imm = Imm::new(sz, *v, !matches!(op, Op::Mov | Op::Test))?;
                    let opc = match (op, imm) {
                        (Op::Mov, _) => op8(sz, 0xc6),
                        (Op::Test, _) => op8(sz, 0xf6),
                        (_, Imm::I8(_)) => 0x83,
                        _ => op8(sz, 0x80),
                    };
                    modrm(Some(sz), &[opc], Field::Digit(digit), Rm::Mem(m), imm)
                }
                _ => None,
            }
        }

        (Op::Not | Op::Idiv, [a]) => {
            let digit = if op == Op::Not { 2 } else { 7 };
            let (sz, rm) = match a {
                O::Reg(r) => (reg_size(*r)?, Rm::Reg(*r)),
                O::Mem(m) => (mem_size(m)?, Rm::Mem(m)),
                _ => return None,
            };
            modrm(Some(sz), &[op8(sz, 0xf6)], Field::Digit(digit), rm, Imm::None)
        }

        (Op::Shl | Op::Shr, [a, O::Imm(1)]) => {
            let digit = if op == Op::Shl { 4 } else { 5 };
            let (sz, rm) = match a {
                O::Reg(r) => (reg_size(*r)?, Rm::Reg(*r)),
                O::Mem(m) => (mem_size(m)?, Rm::Mem(m)),
                _ => return None,
            };
            modrm(Some(sz), &[op8(sz, 0xd0)], Field::Digit(digit), rm, Imm::None)
        }

        (Op::Imul, [O::Reg(x), b]) => {
            let sz = reg_size(*x).filter(|s| *s != Sz::B)?;
            match b {
                O::Reg(y) if reg_size(*y) == Some(sz) => {
                    modrm(Some(sz), &[0x0f, 0xaf], Field::Reg(*x), Rm::Reg(*y), Imm::None)
                }
                O::Mem(m) if m.size.is_none() || mem_size(m) == Some(sz) => {
                    modrm(Some(sz), &[0x0f, 0xaf], Field::Reg(*x), Rm::Mem(m), Imm::None)
                }
                O::Imm(v) => {
                    let imm = Imm::new(sz, *v, true)?;
                    let opc = if matches!(imm, Imm::I8(_)) { 0x6b } else { 0x69 };
                    modrm(Some(sz), &[opc], Field::Reg(*x), Rm::Reg(*x), imm)
                }
                _ => None,
            }
        }

        (Op::Push, [O::Reg(r @ Reg::R64(_))]) => plus_reg(None, 0x50, *r, Imm::None),
        (Op::Pop, [O::Reg(r @ Reg::R64(_))]) => plus_reg(None, 0x58, *r, Imm::None),
        (Op::Push, [O::Imm(v)]) if fits_i32(*v) => {
            let mut out = Vec::new();
            match Imm::new(Sz::D, *v, true)? {
                imm @ Imm::I8(_) => (out.push(0x6a), imm.put(&mut out)),
                imm => (out.push(0x68), imm.put(&mut out)),
            };
            Some(Insn { bytes: out, rip: None })
        }
        (Op::Push | Op::Pop, [O::Mem(m)]) if matches!(m.size, None | Some(Size::Qword)) => {
            let (opc, digit) = if op == Op::Push { (0xff, 6) } else { (0x8f, 0) };
            modrm(None, &[opc], Field::Digit(digit), Rm::Mem(m), Imm::None)
        }

        (Op::Lea, [O::Reg(x), O::Mem(m)]) => {
            let sz = reg_size(*x).filter(|s| *s != Sz::B)?;
            modrm(Some(sz), &[0x8d], Field::Reg(*x), Rm::Mem(m), Imm::None)
        }

        (Op::Movups, [O::Reg(x @ Reg::Xmm(_)), O::Mem(m)]) if m.size.is_none() => {
            modrm(None, &[0x0f, 0x10], Field::Reg(*x), Rm::Mem(m), Imm::None)
        }
        (Op::Movups, [O::Mem(m), O::Reg(x @ Reg::Xmm(_))]) if m.size.is_none() => {
            modrm(None, &[0x0f, 0x11], Field::Reg(*x), Rm::Mem(m), Imm::None)
        }

        (Op::Rdrand, [O::Reg(r)]) => {
            let sz = reg_size(*r).filter(|s| *s != Sz::B)?;
            modrm(Some(sz), &[0x0f, 0xc7], Field::Digit(6), Rm::Reg(*r), Imm::None)
        }

        _ => None,
    }
}

/// Branch kinds, see [`branch`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BranchKind {
    Jmp,
    Jcc(Cond),
    Call,
}

impl BranchKind {
    /// Encoded length (`long`: rel32, else rel8; calls are always rel32).
    pub fn len(self, long: bool) -> u64 {
        match (self, long) {
            (BranchKind::Call, _) | (BranchKind::Jmp, true) => 5,
            (BranchKind::Jcc(_), true) => 6,
            (_, false) => 2,
        }
    }
}

/// A relative branch; `rel` is relative to the end of the instruction and must fit.
pub(crate) fn branch(kind: BranchKind, long: bool, rel: i32) -> Vec<u8> {
    let cc = |c: Cond| match c {
        Cond::E | Cond::Z => 0x4,
        Cond::L => 0xc,
        Cond::Ge => 0xd,
        Cond::G => 0xf,
    };
    let mut out = match (kind, long) {
        (BranchKind::Call, _) => vec![0xe8],
        (BranchKind::Jmp, true) => vec![0xe9],
        (BranchKind::Jcc(c), true) => vec![0x0f, 0x80 | cc(c)],
        (BranchKind::Jmp, false) => vec![0xeb],
        (BranchKind::Jcc(c), false) => vec![0x70 | cc(c)],
    };
    if kind.len(long) == 2 {
        out.push(rel as i8 as u8);
    } else {
        out.extend_from_slice(&rel.to_le_bytes());
    }
    out
}

/// `call`/`jmp` through a 64-bit register or a memory operand (`ff /2`, `ff /4`), e.g.
/// `call rax` or `jmp [rip + slot]`. The translator never emits these; the object writers'
/// entry stubs and import thunks do.
pub(crate) fn encode_indirect(kind: BranchKind, target: &Operand) -> Result<Encoded, String> {
    let digit = match kind {
        BranchKind::Call => Some(2),
        BranchKind::Jmp => Some(4),
        BranchKind::Jcc(_) => None,
    };
    let insn = digit.and_then(|d| match target {
        Operand::Reg(r @ Reg::R64(_)) => modrm(None, &[0xff], Field::Digit(d), Rm::Reg(*r), Imm::None),
        Operand::Mem(m) if matches!(m.size, None | Some(Size::Qword)) => {
            modrm(None, &[0xff], Field::Digit(d), Rm::Mem(m), Imm::None)
        }
        _ => None,
    });
    insn.map(|i| Encoded { bytes: i.bytes, rip: i.rip })
        .ok_or_else(|| format!("internal error: unsupported indirect {kind:?} to {target:?}"))
}

/// `lea dst, [base + index * scale + disp]` with 64-bit registers ([`Mem`] has no index
/// register; the ELF entry stub needs one). `index` can't be `rsp`; `scale` is 1, 2, 4 or 8.
pub(crate) fn encode_lea_indexed(dst: Reg, base: Reg, index: Reg, scale: u8, disp: i32) -> Result<Encoded, String> {
    let (Reg::R64(d), Reg::R64(b), Reg::R64(i)) = (dst, base, index) else {
        return Err(format!("internal error: lea needs 64-bit registers, not {dst:?}, {base:?}, {index:?}"));
    };
    let ss = [1, 2, 4, 8].iter().position(|&s| s == scale).filter(|_| i != 4);
    let Some(ss) = ss else {
        return Err(format!("internal error: unsupported lea index {index:?} * {scale}"));
    };
    let md = if disp == 0 && b & 7 != 5 {
        0x00
    } else if fits_i8(disp as i64) {
        0x40
    } else {
        0x80
    };
    let rex = 0x48 | (d >> 3) << 2 | (i >> 3) << 1 | b >> 3;
    let mut bytes = vec![rex, 0x8d, md | (d & 7) << 3 | 4, (ss as u8) << 6 | (i & 7) << 3 | (b & 7)];
    match md {
        0x40 => bytes.push(disp as u8),
        0x80 => bytes.extend_from_slice(&disp.to_le_bytes()),
        _ => {}
    }
    Ok(Encoded { bytes, rip: None })
}
