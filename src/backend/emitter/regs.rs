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

#[cfg(test)]
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

/// Whether gas reads the bare identifier `name` as a symbol when it is a `call`/`jmp` operand
/// (and not as a register, the `flat` pseudo register, an operator like `and`/`not` or a
/// constant like `byte`/`near`). Mirrors `gas_misreads_symbol` in the translator, which passes
/// such names as [`super::BranchTarget::Quoted`].
pub fn is_symbol(name: &str) -> bool {
    let l = name.to_ascii_lowercase();
    // `<prefix>N` with min <= N <= max, without leading zeros
    let numbered = |prefix: &str, min: u32, max: u32| {
        l.strip_prefix(prefix).is_some_and(|n| {
            !n.is_empty()
                && (n.len() == 1 || !n.starts_with('0'))
                && n.chars().all(|c| c.is_ascii_digit())
                && n.parse::<u32>().is_ok_and(|v| (min..=max).contains(&v))
        })
    };
    // APX registers r16..r31 (+ w/d/b forms)
    let apx = ["", "w", "d", "b"].iter().any(|suffix| {
        let base = l.strip_suffix(suffix).unwrap_or("");
        base.strip_prefix('r').is_some_and(|n| {
            !n.is_empty()
                && !n.starts_with('0')
                && n.chars().all(|c| c.is_ascii_digit())
                && n.parse::<u32>().is_ok_and(|v| (16..=31).contains(&v))
        })
    });
    !(parse(&l).is_some()
        || apx
        || numbered("xmm", 0, 31)
        || numbered("ymm", 0, 31)
        || numbered("zmm", 0, 31)
        || numbered("tmm", 0, 7)
        || numbered("mm", 0, 7)
        || numbered("k", 0, 7)
        || numbered("cr", 0, 15)
        || numbered("dr", 0, 15)
        || matches!(
            l.as_str(),
            "st" | "es" | "cs" | "ss" | "ds" | "fs" | "gs" | "rip" | "eip" | "flat"
                | "and" | "or" | "xor" | "mod" | "shl" | "shr" | "eq" | "ne" | "lt" | "le" | "gt"
                | "ge" | "not" | "byte" | "word" | "dword" | "fword" | "qword" | "mmword" | "tbyte"
                | "oword" | "xmmword" | "ymmword" | "zmmword" | "offset" | "short" | "near" | "far"
        ))
}
