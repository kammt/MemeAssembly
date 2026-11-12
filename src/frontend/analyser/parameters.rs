//! Classifies the parameters of the commands and checks that they are allowed (bully mode
//! replaces what is not).

use crate::backend::emitter::Reg;
use crate::frontend::commands::*;
use crate::frontend::log::{
    format_c, print_debug, print_error, print_error_fmt, print_error_with_key, print_nice_ascii, print_note, ErrorKey,
};
use crate::frontend::state::{CmdLoc, CompileMode, CompileState};
use std::num::IntErrorKind;

/// A decimal number parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Decimal {
    /// The value, saturated to `i64::MIN`/`i64::MAX` if it does not fit into 64 bits.
    value: i64,
    /// Whether the number does not fit into 64 bits.
    overflow: bool,
}

/// Parses a decimal number: optional leading ASCII whitespace, an optional sign and digits,
/// nothing else. The empty string counts as 0 (bully mode can shorten a number to nothing).
fn parse_decimal(s: &[u8]) -> Option<Decimal> {
    if s.is_empty() {
        return Some(Decimal { value: 0, overflow: false });
    }
    let s = s.trim_ascii_start();
    let digits = s.strip_prefix(b"-").or_else(|| s.strip_prefix(b"+")).unwrap_or(s);
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    let text = std::str::from_utf8(s).ok()?;
    match text.parse::<i64>() {
        Ok(value) => Some(Decimal { value, overflow: false }),
        Err(e) => match e.kind() {
            IntErrorKind::PosOverflow => Some(Decimal { value: i64::MAX, overflow: true }),
            IntErrorKind::NegOverflow => Some(Decimal { value: i64::MIN, overflow: true }),
            _ => None,
        },
    }
}

/// The value of a decimal number parameter, 0 if `s` is not one (e.g. only a sign is left
/// after bully mode shortened it).
fn decimal_value(s: &[u8]) -> i64 {
    parse_decimal(s).map_or(0, |d| d.value)
}

fn decimal(text: Vec<u8>) -> Param {
    Param::Decimal { value: decimal_value(&text), text }
}

/// The names of the `PARAM_*` types, by bit.
const PARAM_NAMES: [&str; 8] = [
    "64 bit Register",
    "32 bit Register",
    "16 bit Register",
    "8 bit Register",
    "decimal number",
    "character",
    "monke jump marker name",
    "function name",
];

fn param_name(param_type: u8) -> &'static str {
    PARAM_NAMES.get(param_type.trailing_zeros() as usize).copied().unwrap_or("invalid")
}

/// The registers of each size in the C code's order, from which bully mode picks by index.
const REGISTERS: [(u8, &[&str]); 4] = [
    (PARAM_REG64, &["rax", "rbx", "rcx", "rdx", "rdi", "rsi", "rsp", "rbp", "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15"]),
    (PARAM_REG32, &["eax", "ebx", "ecx", "edx", "edi", "esi", "esp", "ebp", "r8d", "r9d", "r10d", "r11d", "r12d", "r13d", "r14d", "r15d"]),
    (PARAM_REG16, &["ax", "bx", "cx", "dx", "di", "si", "sp", "bp", "r8w", "r9w", "r10w", "r11w", "r12w", "r13w", "r14w", "r15w"]),
    (PARAM_REG8, &["al", "ah", "bl", "bh", "cl", "ch", "dl", "dh", "dil", "sil", "spl", "bpl", "r8b", "r9b", "r10b", "r11b", "r12b", "r13b", "r14b", "r15b"]),
];

/// 8 bit registers without ah, bh, ch and dh.
const NON_HIGH_8_BIT: [&str; 16] =
    ["al", "bl", "cl", "dl", "dil", "sil", "spl", "bpl", "r8b", "r9b", "r10b", "r11b", "r12b", "r13b", "r14b", "r15b"];

/// Escape sequences: (parameter, character literal for gas, value).
const ESCAPE_SEQUENCES: [(&str, &str, u8); 10] = [
    ("\\n", "'\\n'", b'\n'),
    ("\\s", "' '", b' '),
    ("space", "' '", b' '),
    ("\\t", "'\\t'", b'\t'),
    ("\\f", "'\\f'", 0x0C),
    ("\\b", "'\\b'", 0x08),
    ("\\v", "'\\v'", 0x0B),
    ("\\\"", "'\\\"'", b'"'),
    ("\\?", "'\\?'", b'?'),
    ("\\\\", "'\\\\'", b'\\'),
];

fn register_bits(param_type: u8) -> u32 {
    match param_type {
        PARAM_REG8 => 8,
        PARAM_REG16 => 16,
        PARAM_REG32 => 32,
        PARAM_REG64 => 64,
        _ => 0,
    }
}

fn register(name: &str) -> Param {
    Param::Reg(Reg::parse(name).expect("register names of the analyser are valid"))
}

/// Bully mode: a register of the given size, chosen by `computed_index`.
fn random_register(param_type: u8, computed_index: u64) -> Param {
    let (_, names) = REGISTERS.iter().find(|(t, _)| *t == param_type).expect("a register type");
    register(names[(computed_index % names.len() as u64) as usize])
}

/// Bully mode: a decimal number below `modulus`, chosen by `computed_index`.
fn random_decimal(computed_index: u64, modulus: u32) -> Param {
    decimal(((computed_index as u32) % modulus).to_string().into_bytes())
}

/// Bully mode: cut the decimal parameter off after `cut` digits, and further until it is a
/// sign-extended 32 bit number. (The C code wrote a NUL at index `computedIndex % 32`, out of
/// bounds when the number is shorter, and kept numbers that were still too large, which gas
/// then rejected.)
fn shorten_decimal(param: &mut Option<Param>, cut: usize) {
    if let Some(Param::Decimal { value, text }) = param {
        text.truncate(cut);
        while !fits_sign_extended_32(decimal_value(text)) {
            text.pop();
        }
        *value = decimal_value(text);
    }
}

/// What a parameter was recognised as.
enum Class {
    Valid(Param),
    /// A decimal number that does not fit into 64 bits
    TooLarge,
    Invalid,
}

fn as_register(text: &[u8], allowed: u8) -> Option<Param> {
    // (`Reg::parse` ignores the case, the parameters do not)
    let reg = Reg::parse(std::str::from_utf8(text).ok()?).filter(|r| r.name().as_bytes() == text)?;
    let param = Param::Reg(reg);
    (allowed & param.param_type() != 0).then_some(param)
}

fn as_char(text: &[u8]) -> Option<Param> {
    if let Some(&(_, literal, value)) = ESCAPE_SEQUENCES.iter().find(|(e, _, _)| e.as_bytes() == text) {
        // gas has no `\v` escape and reads 'v'
        return Some(Param::Char { value, text: literal.as_bytes().to_vec(), verbatim: text != b"\\v" });
    }
    if let &[c] = text {
        // gas reads `'\'` as an escaped quote
        return Some(Param::Char { value: c, text: vec![b'\'', c, b'\''], verbatim: c != b'\\' });
    }
    // An ASCII code. gas reads numbers with leading zeros as octal.
    let value = parse_decimal(text).filter(|d| (0..=255).contains(&d.value))?.value;
    let digits = text.strip_prefix(b"-").or(text.strip_prefix(b"+")).unwrap_or(text);
    let verbatim = !(digits.len() > 1 && digits[0] == b'0');
    Some(Param::Char { value: value as u8, text: text.to_vec(), verbatim })
}

fn is_monke_label(text: &[u8]) -> bool {
    text.iter().all(|c| matches!(c, b'a' | b'u')) && text.contains(&b'a') && text.contains(&b'u')
}

fn is_function_name(text: &[u8]) -> bool {
    let valid = |c: &u8| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'$' | b'.');
    text.first().is_some_and(|c| !c.is_ascii_digit()) && text.iter().all(valid)
}

/// Classifies the parameter `raw` (as the parser read it) as the first of the `allowed` types it
/// matches.
fn classify(raw: &Param, allowed: u8, target: crate::target::Target) -> Class {
    let Param::Raw { text: raw_text, mangle } = raw else { unreachable!("parameters are classified once") };
    // (validated with the macOS `_` prefix, which is added again when generating code)
    let text = &*raw.text(target);
    if let Some(reg) = as_register(text, allowed) {
        return Class::Valid(reg);
    }
    if allowed & PARAM_DECIMAL != 0 {
        if let Some(d) = parse_decimal(text) {
            return if d.overflow {
                Class::TooLarge
            } else {
                Class::Valid(Param::Decimal { value: d.value, text: text.to_vec() })
            };
        }
    }
    if allowed & PARAM_CHAR != 0 {
        if let Some(c) = as_char(text) {
            return Class::Valid(c);
        }
    }
    if allowed & PARAM_MONKE_LABEL != 0 && is_monke_label(text) {
        return Class::Valid(Param::MonkeLabel(text.to_vec()));
    }
    // (the name itself must not be empty, also when it is validated with the `_` prefix)
    if allowed & PARAM_FUNC_NAME != 0 && !raw_text.is_empty() && is_function_name(text) {
        return Class::Valid(Param::FuncName { name: raw_text.clone(), mangle: *mangle });
    }
    Class::Invalid
}

/// Bully mode: a valid replacement for the invalid parameter `text`, chosen by the global
/// `computed_index`.
fn bully_random_param(text: &[u8], allowed: u8, state: &mut CompileState) -> Param {
    for &c in text {
        state.computed_index = state.computed_index.wrapping_mul(u64::from(c)) / 2;
    }
    let ci = state.computed_index;

    // Choose a parameter type. If it is not allowed, rotate the bitmask until we find a valid one
    let mut chosen: u8 = 1 << (ci % 8);
    while allowed & chosen == 0 {
        chosen = chosen.rotate_left(1);
    }

    match chosen {
        PARAM_DECIMAL => random_decimal(ci, 128),
        PARAM_CHAR => {
            let value = (ci % 128) as u8;
            Param::Char { value, text: value.to_string().into_bytes(), verbatim: true }
        }
        PARAM_MONKE_LABEL => {
            let length = (ci % 7 + 2) as usize;
            let mut label = Vec::with_capacity(length);
            for _ in 0..length {
                label.push(if state.computed_index.is_multiple_of(2) { b'u' } else { b'a' });
                state.computed_index = state.computed_index.wrapping_mul(3) / 2;
            }
            Param::MonkeLabel(label)
        }
        // (without the macOS `_` prefix)
        PARAM_FUNC_NAME => Param::FuncName {
            name: FUNCTION_NAMES[(ci % FUNCTION_NAMES.len() as u64) as usize].as_bytes().to_vec(),
            mangle: false,
        },
        register_type => random_register(register_type, ci),
    }
}

/// The error for a parameter of a type that cannot be a pointer.
fn not_a_pointer_error(param: &Param) -> Option<&'static str> {
    match param {
        Param::Decimal { .. } => Some("a decimal number cannot be a pointer"),
        Param::Char { .. } => Some("a character cannot be a pointer"),
        Param::MonkeLabel(_) => Some("a jump marker cannot be a pointer"),
        Param::FuncName { .. } => Some("a function name cannot be a pointer"),
        Param::Raw { .. } | Param::Reg(_) => None,
    }
}

/// Classifies parameter `i` of `cmd` and reports an invalid one (bully mode replaces it).
fn check_parameter(cmd: &mut ParsedCommand, i: usize, input_file_name: &[u8], state: &mut CompileState) {
    let allowed = cmd.opcode.command().allowed_param_types[i];
    let raw = cmd.parameters[i].clone().expect("the parser fills every used parameter");
    let text = raw.text(state.target).into_owned();
    let shown = String::from_utf8_lossy(&text).into_owned();
    let line = cmd.line_num;

    let class = classify(&raw, allowed, state.target);
    let error = match &class {
        Class::Valid(param) => not_a_pointer_error(param),
        Class::TooLarge => not_a_pointer_error(&Param::Decimal { value: 0, text: Vec::new() }),
        Class::Invalid => None,
    };
    if let Some(message) = error.filter(|_| cmd.is_pointer(i)) {
        if state.bully() {
            // Well then it's not a pointer :shrug:
            cmd.pointer = None;
        } else {
            print_error(state, input_file_name, line, message);
        }
    }

    match class {
        Class::Valid(param) => {
            print_debug(state.log_level, format!("line {line}: {shown:?} is a {}", param_name(param.param_type())));
            if matches!(param, Param::Decimal { value: 69 | 420, .. }) {
                print_nice_ascii();
            }
            cmd.parameters[i] = Some(param);
        }
        Class::TooLarge if state.bully() => {
            print_debug(state.log_level, format!("line {line}: {shown:?} does not fit into 64 bits, bully mode replaces it"));
            // The same replacement as for numbers that don't fit into a register
            cmd.parameters[i] = Some(random_decimal(state.computed_index, 256));
        }
        Class::TooLarge => {
            let format = "invalid parameter provided: \"%s\" does not fit into 64 Bits";
            print_error_fmt(state, input_file_name, line, format, &[&text]);
        }
        Class::Invalid if state.bully() => {
            let param = bully_random_param(&text, allowed, state);
            let new = String::from_utf8_lossy(&param.text(state.target)).into_owned();
            print_debug(state.log_level, format!("line {line}: {shown:?} is invalid, bully mode replaces it with {new:?}"));
            if cmd.is_pointer(i) {
                cmd.pointer = None;
            }
            cmd.parameters[i] = Some(param);
        }
        Class::Invalid => {
            print_debug(state.log_level, format!("line {line}: {shown:?} is invalid"));
            print_error_fmt(state, input_file_name, line, "invalid parameter provided: \"%s\"", &[&text]);
            if state.compile_mode != CompileMode::Obfuscated {
                let names: Vec<&str> = (0..8).filter(|b| allowed & (1 << b) != 0).map(|b| PARAM_NAMES[b]).collect();
                print_note(&format!("the following parameter types are allowed: {}", names.join(", ")), true);
            }
        }
    }
}

/// The number of bits a decimal number needs in a register: two's complement for negative
/// numbers, unsigned for positive ones (so 255 fits into an 8 bit register).
fn bits_needed(number: i64) -> u32 {
    let bits = if number >= 0 { 64 - number.leading_zeros() } else { 65 - (!number).leading_zeros() };
    bits.max(1)
}

/// The checks for commands with two parameters.
fn check_combination(cmd: &mut ParsedCommand, input_file_name: &[u8], state: &mut CompileState) {
    let target = state.target;
    let line = cmd.line_num;
    let (t0, t1) = (cmd.param_type(0), cmd.param_type(1));

    // 1: If a number and a register are used, the number must fit in the register
    // 2: If a number and a 64 Bit register are used (and the command is not a mov-command),
    //    the number must be sign-extendable from 32 Bits
    if (t0 == PARAM_DECIMAL && param_is_reg(t1)) || (t1 == PARAM_DECIMAL && param_is_reg(t0)) {
        let decimal_index = if t0 == PARAM_DECIMAL { 0 } else { 1 };
        let reg_index = 1 - decimal_index;
        let reg_size = register_bits(cmd.param_type(reg_index));
        let Some(Param::Decimal { value: number, .. }) = cmd.parameters[decimal_index] else { unreachable!() };
        let bits_needed = bits_needed(number);

        if bits_needed > reg_size {
            if state.bully() {
                // We replace the number with something that is guaranteed to fit into all registers
                cmd.parameters[decimal_index] = Some(random_decimal(state.computed_index, 256));
            } else {
                let format = "invalid parameter combination: '%s' (%u bits) does not fit into register '%s' of size %u";
                let (number, register) = (cmd.param_text(decimal_index, target), cmd.param_text(reg_index, target));
                let (bits, size) = (bits_needed.to_string(), reg_size.to_string());
                let message = format_c(format, &[&number, bits.as_bytes(), &register, size.as_bytes()]);
                // (the C code passed 3 as the argument count)
                print_error_with_key(state, input_file_name, line, ErrorKey::new(format, 3), &message);
            }
        } else if cmd.opcode != Opcode::Mov && reg_size == 64 && !fits_sign_extended_32(number) {
            if state.bully() {
                // We just make the number shorter than 32 bits :bigBrain:
                shorten_decimal(&mut cmd.parameters[decimal_index], (state.computed_index % 32) as usize);
                // Also, change computedIndex. Just because
                state.computed_index = state.computed_index.wrapping_add((number & 0xFFF) as u64);
            } else {
                print_error(state, input_file_name, line, "invalid parameter combination: 64 Bit arithmetic operation commands require the decimal number to be sign-extendable from 32 Bits");
            }
        }
    }

    // 3: If two registers are used, they must be of the same size
    let register_operand = |i: usize| param_is_reg(cmd.param_type(i)) && !cmd.is_pointer(i);
    if register_operand(0) && register_operand(1) && t0 != t1 {
        if state.bully() {
            cmd.parameters[1] = Some(random_register(t0, state.computed_index));
        } else {
            print_error(state, input_file_name, line, "invalid parameter combination: cannot combine registers of different size");
        }
    }

    // ah/bh/ch/dh can't be encoded together with a register that needs a REX prefix: use
    // another 8 bit register instead
    if state.bully() {
        if let Some((high, _)) = high_byte_conflict(cmd) {
            let index = (state.computed_index % NON_HIGH_8_BIT.len() as u64) as usize;
            cmd.parameters[high] = Some(register(NON_HIGH_8_BIT[index]));
        }
    }

    // 4: If there's a pointer parameter, the other parameter must be a register, otherwise the
    // operand size is unknown. Skipped in bully mode (handled by the translator).
    if !state.bully() && cmd.pointer.is_some_and(|p| !param_is_reg(cmd.param_type(1 - p))) {
        print_error(state, input_file_name, line, "invalid parameter combination: operand size unknown");
    }
}

/// Combinations the C compiler accepted, but that can't be assembled.
fn check_encodable(cmd: &ParsedCommand, input_file_name: &[u8], state: &mut CompileState, push_too_large: bool) {
    let line = cmd.line_num;
    if let Some(problem) = pointer_problem(cmd) {
        let message = match problem {
            PointerProblem::Base => "invalid parameter combination: a pointer must be a 64 or 32 bit register",
            PointerProblem::Unsupported => "invalid parameter combination: this command cannot use a pointer here",
        };
        print_error(state, input_file_name, line, message);
    } else if cmd.pointer.is_some_and(|p| pointer_use(cmd.opcode, p) == PointerUse::NeedsSize) {
        print_error(state, input_file_name, line, "invalid parameter combination: operand size unknown");
    } else if let Some((high, other)) = high_byte_conflict(cmd) {
        let format = "invalid parameter combination: '%s' cannot be combined with '%s'";
        let (high, other) = (cmd.param_text(high, state.target), cmd.param_text(other, state.target));
        print_error_fmt(state, input_file_name, line, format, &[&high, &other]);
    } else if push_too_large {
        print_error(state, input_file_name, line, "invalid parameter combination: pushed decimal numbers must be sign-extendable from 32 Bits");
    }
}

/// Checks if the given parsed command adheres to the parameter constraints (i.e. the parameters
/// are legal).
pub fn check_parameters(loc: CmdLoc, input_file_name: &[u8], state: &mut CompileState) {
    // (a copy, as reporting an error needs the state)
    let mut cmd = state.cmd(loc).clone();
    check_command(&mut cmd, input_file_name, state);
    *state.cmd_mut(loc) = cmd;
}

fn check_command(cmd: &mut ParsedCommand, input_file_name: &[u8], state: &mut CompileState) {
    let used_parameters = cmd.opcode.command().used_parameters();
    let errors_before = state.compiler_errors;

    for i in 0..used_parameters {
        check_parameter(cmd, i, input_file_name, state);
    }

    // Well then it's not a pointer :shrug: (before the combination checks, so that a former
    // pointer gets the size of the other register)
    if state.bully() && pointer_problem(cmd).is_some() {
        cmd.pointer = None;
    }

    if used_parameters == 2 {
        check_combination(cmd, input_file_name, state);
    }

    // `push` only takes sign-extended 32-bit immediates
    let push_too_large = cmd.opcode == Opcode::Push
        && matches!(cmd.parameters[0], Some(Param::Decimal { value, .. }) if !fits_sign_extended_32(value));
    if state.bully() {
        if push_too_large {
            // Shorten it like the "sign-extendable from 32 Bits" fix-up above
            shorten_decimal(&mut cmd.parameters[0], (state.computed_index % 32) as usize);
        }
    } else if state.compiler_errors == errors_before {
        // (only reported if the checks above found nothing wrong with this command)
        check_encodable(cmd, input_file_name, state, push_too_large);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PointerProblem {
    /// `[al]`, `[ax]`: only 64 and 32 bit registers can address memory
    Base,
    /// The generated code can't use a memory operand there (see [`PointerUse`])
    Unsupported,
}

fn pointer_problem(cmd: &ParsedCommand) -> Option<PointerProblem> {
    let index = cmd.pointer?;
    if !matches!(cmd.param_type(index), PARAM_REG64 | PARAM_REG32) {
        return Some(PointerProblem::Base);
    }
    (pointer_use(cmd.opcode, index) == PointerUse::Unsupported).then_some(PointerProblem::Unsupported)
}

/// Registers whose encoding needs a REX prefix (r8-r15 in any size, spl, bpl, sil, dil).
fn needs_rex(register: Reg) -> bool {
    match register {
        Reg::R64(i) | Reg::R32(i) | Reg::R16(i) => i >= 8,
        Reg::R8(i) => i >= 4,
        Reg::R8H(_) | Reg::Xmm(_) => false,
    }
}

/// A two-operand command combining ah/bh/ch/dh with a register that needs a REX prefix
/// (including a pointer's base register): (index of the high byte register, the other index).
fn high_byte_conflict(cmd: &ParsedCommand) -> Option<(usize, usize)> {
    if cmd.opcode.command().used_parameters() != 2 {
        return None;
    }
    let high = (0..2).find(|&i| !cmd.is_pointer(i) && matches!(cmd.parameters[i], Some(Param::Reg(Reg::R8H(_)))))?;
    let other = 1 - high;
    matches!(cmd.parameters[other], Some(Param::Reg(r)) if needs_rex(r)).then_some((high, other))
}

/// Whether a 64 bit immediate can be encoded as a sign-extended 32 bit immediate.
fn fits_sign_extended_32(number: i64) -> bool {
    i32::try_from(number).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::target::Target;

    #[test]
    fn decimals() {
        let d = |s: &[u8]| parse_decimal(s).map(|d| (d.value, d.overflow));
        assert_eq!(d(b"42"), Some((42, false)));
        assert_eq!(d(b" \t-12"), Some((-12, false)));
        assert_eq!(d(b"+7"), Some((7, false)));
        assert_eq!(d(b""), Some((0, false)));
        assert_eq!(d(b"-9223372036854775808"), Some((i64::MIN, false)));
        assert_eq!(d(b"99999999999999999999"), Some((i64::MAX, true)));
        assert_eq!(d(b"-99999999999999999999"), Some((i64::MIN, true)));
        for s in [&b"-"[..], b"+", b" ", b"12x", b"99999999999999999999x", b"1 2", b"0x10", b"--1", b"1\0"] {
            assert_eq!(d(s), None, "{:?}", String::from_utf8_lossy(s));
        }
        assert_eq!(decimal_value(b"-"), 0);
        assert_eq!(decimal_value(b"123"), 123);
    }

    #[test]
    fn bits() {
        let cases = [(0, 1), (1, 1), (255, 8), (256, 9), (-1, 1), (-128, 8), (-129, 9), (i64::MAX, 63), (i64::MIN, 64)];
        for (number, bits) in cases {
            assert_eq!(bits_needed(number), bits, "{number}");
        }
    }

    #[test]
    fn classification() {
        let class = |text: &[u8], allowed| match classify(&Param::Raw { text: text.to_vec(), mangle: false }, allowed, Target::Linux) {
            Class::Valid(p) => Some(p.param_type()),
            Class::TooLarge => Some(0),
            Class::Invalid => None,
        };
        assert_eq!(class(b"rax", PARAM_REG), Some(PARAM_REG64));
        assert_eq!(class(b"ah", PARAM_REG), Some(PARAM_REG8));
        assert_eq!(class(b"RAX", PARAM_REG), None);
        assert_eq!(class(b"xmm0", PARAM_REG), None);
        assert_eq!(class(b"eax", PARAM_REG64), None);
        assert_eq!(class(b"65", PARAM_IMM), Some(PARAM_DECIMAL));
        assert_eq!(class(b"65", PARAM_CHAR), Some(PARAM_CHAR));
        assert_eq!(class(b"256", PARAM_CHAR), None);
        assert_eq!(class(b"99999999999999999999", PARAM_DECIMAL), Some(0));
        assert_eq!(class(b"space", PARAM_CHAR), Some(PARAM_CHAR));
        assert_eq!(class(b"uau", PARAM_MONKE_LABEL), Some(PARAM_MONKE_LABEL));
        for label in [&b"uu"[..], b"", b"uax"] {
            assert_eq!(class(label, PARAM_MONKE_LABEL), None);
        }
        assert_eq!(class(b"_f.$1", PARAM_FUNC_NAME), Some(PARAM_FUNC_NAME));
        for name in [&b"1f"[..], b"", b"f-g"] {
            assert_eq!(class(name, PARAM_FUNC_NAME), None);
        }
    }
}
