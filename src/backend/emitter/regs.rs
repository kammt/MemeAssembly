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
