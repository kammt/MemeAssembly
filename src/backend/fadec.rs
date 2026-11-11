//! The fadec-style encoding interface the object writers use for their fixed startup code and
//! import thunks (`writers::Asm`), implemented in Rust for exactly the mnemonics they need.
//! It is API-compatible with the vendored C fadec encoder it replaced, and was verified to
//! produce identical bytes for every one of these mnemonics. The writers could later move to
//! the emitter's typed encoder instead.
#![allow(non_upper_case_globals, dead_code)]

pub type FeOp = i64;

// Registers
pub const FE_NOREG: FeOp = 0;
pub const FE_AX: FeOp = 0x100;
pub const FE_CX: FeOp = 0x101;
pub const FE_DX: FeOp = 0x102;
pub const FE_BX: FeOp = 0x103;
pub const FE_SP: FeOp = 0x104;
pub const FE_BP: FeOp = 0x105;
pub const FE_SI: FeOp = 0x106;
pub const FE_DI: FeOp = 0x107;
pub const FE_R8: FeOp = 0x108;
pub const FE_R9: FeOp = 0x109;
pub const FE_R10: FeOp = 0x10a;
pub const FE_R11: FeOp = 0x10b;
pub const FE_R12: FeOp = 0x10c;
pub const FE_R13: FeOp = 0x10d;
pub const FE_R14: FeOp = 0x10e;
pub const FE_R15: FeOp = 0x10f;
pub const FE_IP: FeOp = 0x120;

/// General purpose register number `n` (0 = rax ... 15 = r15); the size is selected by the
/// mnemonic.
pub const fn gp(n: u8) -> FeOp {
    0x100 + n as FeOp
}

/// A memory operand `[base + index * scale + off]` (`scale` is 1, 2, 4 or 8, 0 without index).
/// With `base == FE_IP` the offset is relative to the *start* of the instruction.
pub const fn mem(base: FeOp, scale: u8, index: FeOp, off: i32) -> FeOp {
    i64::MIN
        | ((base & 0xfff) << 32)
        | ((index & 0xfff) << 44)
        | (((scale as i64) & 0xf) << 56)
        | ((off as i64) & 0xffff_ffff)
}

/// Force the rel32 form of a jump/call (or-ed into the mnemonic).
pub const FE_JMPL: u64 = 0x1_0000_0000;

/// Operand forms: register (`r`), memory (`m`), immediate (`i`).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Form {
    None,
    R,
    M,
    /// `op r/m, reg`
    RR,
    /// `op reg, r/m`
    RM,
    MR,
    RI,
    MI,
    /// `opcode + reg` (push/pop)
    PlusR,
    Rel,
}

/// A mnemonic: `opcode | digit << 16 | size << 20 | form << 24` (size: operand bytes).
const fn m(opcode: u16, digit: u8, size: u8, form: Form) -> u64 {
    opcode as u64 | (digit as u64) << 16 | (size as u64) << 20 | (form as u64) << 24
}

pub const FE_XOR32rr: u64 = m(0x31, 0, 4, Form::RR);
pub const FE_MOV32rr: u64 = m(0x89, 0, 4, Form::RR);
pub const FE_MOV64rr: u64 = m(0x89, 0, 8, Form::RR);
pub const FE_SUB64rr: u64 = m(0x29, 0, 8, Form::RR);
pub const FE_SBB32rr: u64 = m(0x19, 0, 4, Form::RR);
pub const FE_MOV32rm: u64 = m(0x8b, 0, 4, Form::RM);
pub const FE_MOV64rm: u64 = m(0x8b, 0, 8, Form::RM);
pub const FE_LEA64rm: u64 = m(0x8d, 0, 8, Form::RM);
pub const FE_MOV64mr: u64 = m(0x89, 0, 8, Form::MR);
pub const FE_ADD64ri: u64 = m(0x81, 0, 8, Form::RI);
pub const FE_AND64ri: u64 = m(0x81, 4, 8, Form::RI);
pub const FE_SUB64ri: u64 = m(0x81, 5, 8, Form::RI);
pub const FE_MOV32ri: u64 = m(0xb8, 0, 4, Form::RI);
pub const FE_MOV64ri: u64 = m(0xc7, 0, 8, Form::RI);
pub const FE_MOV8mi: u64 = m(0xc6, 0, 1, Form::MI);
pub const FE_MOV32mi: u64 = m(0xc7, 0, 4, Form::MI);
pub const FE_MOV64mi: u64 = m(0xc7, 0, 8, Form::MI);
pub const FE_NEG32r: u64 = m(0xf7, 3, 4, Form::R);
pub const FE_NEG64r: u64 = m(0xf7, 3, 8, Form::R);
pub const FE_CALLr: u64 = m(0xff, 2, 0, Form::R);
pub const FE_CALLm: u64 = m(0xff, 2, 0, Form::M);
pub const FE_JMPm: u64 = m(0xff, 4, 0, Form::M);
pub const FE_PUSHr: u64 = m(0x50, 0, 0, Form::PlusR);
pub const FE_POPr: u64 = m(0x58, 0, 0, Form::PlusR);
pub const FE_RET: u64 = m(0xc3, 0, 0, Form::None);
pub const FE_HLT: u64 = m(0xf4, 0, 0, Form::None);
pub const FE_INT3: u64 = m(0xcc, 0, 0, Form::None);
pub const FE_SYSCALL: u64 = m(0x050f, 0, 0, Form::None);
pub const FE_CALL: u64 = m(0xe8, 0, 0, Form::Rel);
pub const FE_JMP: u64 = m(0xe9, 0, 0, Form::Rel);

fn reg(op: FeOp) -> Result<u8, ()> {
    if (0x100..0x110).contains(&op) {
        Ok((op - 0x100) as u8)
    } else {
        Err(())
    }
}

fn fits_i8(v: i64) -> bool {
    (-128..=127).contains(&v)
}

/// ModRM (+ SIB, displacement) for `rm` with `reg` field `r`; the REX bits it needs; and for
/// RIP-relative operands the offset relative to the instruction start.
fn modrm(r: u8, rm: FeOp) -> Result<(Vec<u8>, u8, Option<i32>), ()> {
    if rm >= 0 {
        let n = reg(rm)?;
        return Ok((vec![0xc0 | (r & 7) << 3 | (n & 7)], n >> 3, None));
    }
    let base = (rm >> 32) & 0xfff;
    let index = (rm >> 44) & 0xfff;
    let scale = ((rm >> 56) & 0xf) as u8;
    let off = rm as i32;
    if base == FE_IP {
        return Ok((vec![0x05 | (r & 7) << 3, 0, 0, 0, 0], 0, Some(off)));
    }
    let mut out = Vec::new();
    let b = reg(base)?;
    let (md, disp) = match off {
        0 if b & 7 != 5 => (0x00, vec![]),
        d if fits_i8(d as i64) => (0x40, vec![d as u8]),
        d => (0x80, d.to_le_bytes().to_vec()),
    };
    let mut rex = b >> 3;
    if index != FE_NOREG || b & 7 == 4 {
        let (x, ss) = if index == FE_NOREG {
            (4, 0)
        } else {
            (reg(index)?, [1, 2, 4, 8].iter().position(|&s| s == scale).ok_or(())? as u8)
        };
        rex |= (x >> 3) << 1;
        out.extend([md | (r & 7) << 3 | 4, ss << 6 | (x & 7) << 3 | (b & 7)]);
    } else {
        out.push(md | (r & 7) << 3 | (b & 7));
    }
    out.extend(disp);
    Ok((out, rex, None))
}

/// Encode one instruction (immediates as plain values, registers as `FE_*`/`gp()`, memory
/// operands via `mem()`).
pub fn encode(mnem: u64, ops: &[FeOp]) -> Result<Vec<u8>, ()> {
    let opcode = (mnem & 0xffff) as u16;
    let digit = ((mnem >> 16) & 0xf) as u8;
    let size = (mnem >> 20) & 0xf;
    let form = (mnem >> 24) & 0xff;
    let op = |i: usize| ops.get(i).copied().ok_or(());
    let w = if size == 8 { 8 } else { 0 };
    let mut out = Vec::new();
    let mut imm: Vec<u8> = Vec::new();
    let (field, rm) = match form {
        f if f == Form::None as u64 => return Ok(opcode.to_le_bytes()[..if opcode > 0xff { 2 } else { 1 }].to_vec()),
        f if f == Form::PlusR as u64 => {
            let n = reg(op(0)?)?;
            if n >= 8 {
                out.push(0x41);
            }
            out.push(opcode as u8 + (n & 7));
            return Ok(out);
        }
        f if f == Form::R as u64 || f == Form::M as u64 => (digit, op(0)?),
        f if f == Form::RR as u64 || f == Form::MR as u64 => (reg(op(1)?)?, op(0)?),
        f if f == Form::RM as u64 => (reg(op(0)?)?, op(1)?),
        f if f == Form::RI as u64 || f == Form::MI as u64 => {
            let v = op(1)?;
            if opcode == 0xb8 {
                // mov r32, imm32
                let n = reg(op(0)?)?;
                if n >= 8 {
                    out.push(0x41);
                }
                out.push(0xb8 + (n & 7));
                out.extend((v as u32).to_le_bytes());
                return Ok(out);
            }
            if opcode == 0xc7 && form == Form::RI as u64 && i32::try_from(v).is_err() {
                // movabs
                let n = reg(op(0)?)?;
                out.extend([0x48 | n >> 3, 0xb8 + (n & 7)]);
                out.extend(v.to_le_bytes());
                return Ok(out);
            }
            match size {
                1 => imm.push(v as u8),
                _ if opcode == 0x81 && fits_i8(v) => imm.push(v as u8),
                _ => imm.extend(i32::try_from(v).map_err(|_| ())?.to_le_bytes()),
            }
            if opcode == 0x81 && imm.len() == 4 && op(0)? == FE_AX {
                // accumulator form (`add rax, imm32` = `48 05 id`)
                out.extend([0x40 | w, digit << 3 | 5]);
                out.extend(imm);
                return Ok(out);
            }
            (digit, op(0)?)
        }
        _ => return Err(()),
    };
    let opcode = if opcode == 0x81 && imm.len() == 1 { 0x83 } else { opcode as u8 };
    let (modrm, rex_xb, rip) = modrm(field, rm)?;
    let rex = w | (field >> 3) << 2 | rex_xb;
    if rex != 0 {
        out.push(0x40 | rex);
    }
    out.push(opcode);
    let field_at = out.len() + 1;
    out.extend(modrm);
    out.extend(imm);
    if let Some(off) = rip {
        let disp = off.wrapping_sub(out.len() as i32);
        out[field_at..field_at + 4].copy_from_slice(&disp.to_le_bytes());
    }
    Ok(out)
}

/// Encode a relative branch (`FE_CALL`, `FE_JMP`) located at section offset `at`, jumping to
/// section offset `target`. Or `FE_JMPL` into `mnem` to force rel32 for `FE_JMP`.
pub fn encode_branch(mnem: u64, at: u64, target: u64) -> Result<Vec<u8>, ()> {
    let base = mnem & !FE_JMPL;
    let rel = |len: i64| target as i64 - (at as i64 + len);
    if base == FE_JMP && mnem & FE_JMPL == 0 && fits_i8(rel(2)) {
        return Ok(vec![0xeb, rel(2) as u8]);
    }
    let op = match base {
        FE_CALL => 0xe8,
        FE_JMP => 0xe9,
        _ => return Err(()),
    };
    let mut out = vec![op];
    out.extend(i32::try_from(rel(5)).map_err(|_| ())?.to_le_bytes());
    Ok(out)
}
