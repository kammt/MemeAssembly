//! Register names as GNU as (x86-64, Intel syntax) understands them.

use super::Reg;

const R64: [&str; 16] = [
    "rax", "rcx", "rdx", "rbx", "rsp", "rbp", "rsi", "rdi", "r8", "r9", "r10", "r11", "r12", "r13",
    "r14", "r15",
];
const R32: [&str; 16] = [
    "eax", "ecx", "edx", "ebx", "esp", "ebp", "esi", "edi", "r8d", "r9d", "r10d", "r11d", "r12d",
    "r13d", "r14d", "r15d",
];
const R16: [&str; 16] = [
    "ax", "cx", "dx", "bx", "sp", "bp", "si", "di", "r8w", "r9w", "r10w", "r11w", "r12w", "r13w",
    "r14w", "r15w",
];
const R8: [&str; 16] = [
    "al", "cl", "dl", "bl", "spl", "bpl", "sil", "dil", "r8b", "r9b", "r10b", "r11b", "r12b",
    "r13b", "r14b", "r15b",
];
const R8H: [&str; 4] = ["ah", "ch", "dh", "bh"];

/// Parse one of the registers representable as [`Reg`]. Case-insensitive, like gas.
pub fn parse(name: &str) -> Option<Reg> {
    let l = name.to_ascii_lowercase();
    let find = |t: &[&str]| t.iter().position(|r| *r == l).map(|i| i as u8);
    if let Some(i) = find(&R64) {
        return Some(Reg::R64(i));
    }
    if let Some(i) = find(&R32) {
        return Some(Reg::R32(i));
    }
    if let Some(i) = find(&R16) {
        return Some(Reg::R16(i));
    }
    if let Some(i) = find(&R8) {
        return Some(Reg::R8(i));
    }
    if let Some(i) = find(&R8H) {
        return Some(Reg::R8H(i));
    }
    if let Some(n) = l.strip_prefix("xmm") {
        if let Ok(i) = n.parse::<u8>() {
            if i < 16 && (n.len() == 1 || !n.starts_with('0')) {
                return Some(Reg::Xmm(i));
            }
        }
    }
    None
}

pub fn name(r: Reg) -> String {
    match r {
        Reg::R64(i) => R64[i as usize].into(),
        Reg::R32(i) => R32[i as usize].into(),
        Reg::R16(i) => R16[i as usize].into(),
        Reg::R8(i) => R8[i as usize].into(),
        Reg::R8H(i) => R8H[i as usize].into(),
        Reg::Xmm(i) => format!("xmm{i}"),
    }
}

/// How gas interprets a bare identifier used as a `call`/`jmp` operand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ident {
    /// An ordinary symbol.
    Symbol,
    /// A register usable as an indirect branch target (64-bit GPR, or 16-bit GPR with 0x66).
    Gpr(Reg),
    /// APX extended GPR r16..r31 (64-bit form) – encodable by gas only with a REX2 prefix.
    ApxGpr(u8),
    /// Any other register name gas knows (eax, al, xmm0, st, cr0, es, k0, ...).
    OtherReg,
    /// `flat` (a pseudo register in Intel syntax).
    PseudoReg,
    /// Intel-syntax operator words (`and`, `or`, `xor`, `mod`, `shl`, `shr`, `eq`, ...).
    Operator,
    /// `not` (unary operator, gas then fails to parse an expression).
    UnaryOperator,
    /// Intel-syntax size/type keywords that gas evaluates to constants (`byte`, `near`, ...).
    Constant,
}

pub fn classify(name: &str) -> Ident {
    let l = name.to_ascii_lowercase();
    if let Some(r) = parse(&l) {
        return match r {
            Reg::R64(_) | Reg::R16(_) => Ident::Gpr(r),
            _ => Ident::OtherReg,
        };
    }
    // APX registers r16..r31 (+ d/w/b forms).
    if let Some(rest) = l.strip_prefix('r') {
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        let suffix = &rest[digits.len()..];
        if !digits.is_empty() && !digits.starts_with('0') {
            if let Ok(n) = digits.parse::<u8>() {
                if (16..32).contains(&n) {
                    match suffix {
                        "" => return Ident::ApxGpr(n),
                        "d" | "w" | "b" => return Ident::OtherReg,
                        _ => {}
                    }
                }
            }
        }
    }
    let numbered = |prefix: &str, max: u32| -> bool {
        l.strip_prefix(prefix).is_some_and(|n| {
            !n.is_empty()
                && (n.len() == 1 || !n.starts_with('0'))
                && n.chars().all(|c| c.is_ascii_digit())
                && n.parse::<u32>().is_ok_and(|v| v <= max)
        })
    };
    if numbered("xmm", 31)
        || numbered("ymm", 31)
        || numbered("zmm", 31)
        || numbered("tmm", 7)
        || numbered("mm", 7)
        || numbered("k", 7)
        || numbered("cr", 15)
        || numbered("dr", 15)
        || numbered("st", 7) && l != "st"
        || numbered("bnd", 3)
        || numbered("tr", 7)
    {
        return Ident::OtherReg;
    }
    match l.as_str() {
        "st" | "es" | "cs" | "ss" | "ds" | "fs" | "gs" | "rip" | "eip" | "rflags" => Ident::OtherReg,
        "flat" => Ident::PseudoReg,
        "and" | "or" | "xor" | "mod" | "shl" | "shr" | "eq" | "ne" | "lt" | "le" | "gt" | "ge" => {
            Ident::Operator
        }
        "not" => Ident::UnaryOperator,
        "byte" | "word" | "dword" | "fword" | "qword" | "mmword" | "tbyte" | "oword" | "xmmword"
        | "ymmword" | "zmmword" | "offset" | "short" | "near" | "far" => Ident::Constant,
        _ => Ident::Symbol,
    }
}
