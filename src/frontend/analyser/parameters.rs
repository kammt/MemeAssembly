//! Port of compiler/analyser/parameters.c

use crate::backend::emitter::Reg;
use crate::frontend::commands::*;
use crate::frontend::libc::{cstr, decimal_value, parse_decimal};
use crate::frontend::log::{
    print_debug, print_error, print_internal_compiler_error, print_nice_ascii, print_note, Arg,
};

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

pub const REGISTERS_64_BIT: [&str; 16] = [
    "rax", "rbx", "rcx", "rdx", "rdi", "rsi", "rsp", "rbp", "r8", "r9", "r10", "r11", "r12", "r13",
    "r14", "r15",
];
pub const REGISTERS_32_BIT: [&str; 16] = [
    "eax", "ebx", "ecx", "edx", "edi", "esi", "esp", "ebp", "r8d", "r9d", "r10d", "r11d", "r12d",
    "r13d", "r14d", "r15d",
];
pub const REGISTERS_16_BIT: [&str; 16] = [
    "ax", "bx", "cx", "dx", "di", "si", "sp", "bp", "r8w", "r9w", "r10w", "r11w", "r12w", "r13w",
    "r14w", "r15w",
];
pub const REGISTERS_8_BIT: [&str; 20] = [
    "al", "ah", "bl", "bh", "cl", "ch", "dl", "dh", "dil", "sil", "spl", "bpl", "r8b", "r9b",
    "r10b", "r11b", "r12b", "r13b", "r14b", "r15b",
];

const ESCAPE_SEQUENCES: [&str; 10] =
    ["\\n", "\\s", "space", "\\t", "\\f", "\\b", "\\v", "\\\"", "\\?", "\\\\"];
const TRANSLATED_ESCAPE_SEQUENCES: [&str; 10] =
    ["'\\n'", "' '", "' '", "'\\t'", "'\\f'", "'\\b'", "'\\v'", "'\\\"'", "'\\?'", "'\\\\'"];
const ESCAPE_SEQUENCE_VALUES: [u8; 10] = [b'\n', b' ', b' ', b'\t', 0x0C, 0x08, 0x0B, b'"', b'?', b'\\'];

fn get_register_size(param_type: u8) -> u32 {
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

/// Returns a register of the specified size chosen by `computedIndex`.
fn get_random_register(param_type: u8, computed_index: u64) -> Option<Param> {
    let pick = |regs: &[&str]| Some(register(regs[(computed_index % regs.len() as u64) as usize]));
    match param_type {
        PARAM_REG64 => pick(&REGISTERS_64_BIT),
        PARAM_REG32 => pick(&REGISTERS_32_BIT),
        PARAM_REG16 => pick(&REGISTERS_16_BIT),
        PARAM_REG8 => pick(&REGISTERS_8_BIT),
        _ => None,
    }
}

fn decimal(text: Vec<u8>) -> Param {
    Param::Decimal { value: decimal_value(&text), text }
}

/// The value of a decimal parameter (0 for other parameters).
fn decimal_param_value(cmd: &ParsedCommand, i: usize) -> i64 {
    match &cmd.parameters[i] {
        Some(Param::Decimal { value, .. }) => *value,
        _ => 0,
    }
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

fn print_parameter_usage_note(allowed_params: u8) {
    let names: Vec<&str> =
        (0..8).filter(|i| allowed_params & (1 << i) != 0).map(|i| PARAM_NAMES[i]).collect();
    let s = names.join(", ");
    print_note("the following parameter types are allowed: %s", true, &[Arg::S(s.as_bytes())]);
}

/// Prints the "X cannot be a pointer" error (or silently fixes it in bully mode).
fn check_not_pointer(
    cmd: &mut ParsedCommand,
    parameter_num: usize,
    input_file_name: &[u8],
    state: &mut CompileState,
    message: &str,
) {
    if cmd.is_pointer as usize == parameter_num + 1 {
        if state.compile_mode != CompileMode::Bully {
            print_error(state, input_file_name, cmd.line_num as u64, message, 0, &[]);
        } else {
            // Well then it's not a pointer :shrug:
            cmd.is_pointer = 0;
        }
    }
}

/// Checks if the given parsed command adheres to the parameter constraints (i.e. the parameters
/// are legal).
pub fn check_parameters(loc: CmdLoc, input_file_name: &[u8], state: &mut CompileState) {
    let mut cmd = state.cmd(loc).clone();
    check_command(&mut cmd, input_file_name, state);
    *state.cmd_mut(loc) = cmd;
}

fn check_command(cmd: &mut ParsedCommand, input_file_name: &[u8], state: &mut CompileState) {
    let ll = state.log_level;
    let target = state.target;
    print_debug(ll, "Starting parameter validity check", &[]);
    let command = cmd.opcode.command();
    let used_parameters = command.used_parameters as usize;
    let errors_before = state.compiler_errors;

    'params: for parameter_num in 0..used_parameters {
        let parameter = cstr(&cmd.param_text(parameter_num, target)).to_vec();
        print_debug(ll, "\tChecking parameter %s", &[Arg::S(&parameter)]);

        let allowed_types = command.allowed_param_types[parameter_num];

        let reg_checks: [(u8, &[&str], &str, &str); 4] = [
            (PARAM_REG64, &REGISTERS_64_BIT, "\t\tParameter is a 64 bit register", "\t\tParameter is not a 64 bit register"),
            (PARAM_REG32, &REGISTERS_32_BIT, "\t\tParameter is a 32 bit register", "\t\tParameter is not a 32 bit register"),
            (PARAM_REG16, &REGISTERS_16_BIT, "\t\tParameter is a 16 bit register", "\t\tParameter is not a 16 bit register"),
            (PARAM_REG8, &REGISTERS_8_BIT, "\t\tParameter is an 8 bit register", "\t\tParameter is not an 8 bit register"),
        ];
        for (ty, regs, yes, no) in reg_checks {
            if allowed_types & ty != 0 {
                if let Some(&name) = regs.iter().find(|r| r.as_bytes() == parameter) {
                    print_debug(ll, yes, &[]);
                    cmd.parameters[parameter_num] = Some(register(name));
                    continue 'params;
                }
                print_debug(ll, no, &[]);
            }
        }

        if allowed_types & PARAM_DECIMAL != 0 {
            if let Some(r) = parse_decimal(&parameter) {
                print_debug(ll, "\t\tParameter is a decimal number", &[]);
                check_not_pointer(cmd, parameter_num, input_file_name, state, "a decimal number cannot be a pointer");
                if r.value == 69 || r.value == 420 {
                    print_nice_ascii();
                }
                // The C code silently used strtoll's saturated value (INT64_MAX / INT64_MIN)
                if !r.overflow {
                    cmd.parameters[parameter_num] = Some(Param::Decimal { value: r.value, text: parameter });
                } else {
                    if state.compile_mode != CompileMode::Bully {
                        print_error(
                            state,
                            input_file_name,
                            cmd.line_num as u64,
                            "invalid parameter provided: \"%s\" does not fit into 64 Bits",
                            1,
                            &[Arg::S(&parameter)],
                        );
                    } else {
                        // The same replacement as for numbers that don't fit into a register
                        cmd.parameters[parameter_num] =
                            Some(decimal(((state.computed_index as u32) % 256).to_string().into_bytes()));
                    }
                }
                continue 'params;
            }
            print_debug(ll, "\t\tParameter is not a decimal number", &[]);
        }

        if allowed_types & PARAM_CHAR != 0 {
            if let Some(i) = ESCAPE_SEQUENCES.iter().position(|e| e.as_bytes() == parameter) {
                cmd.parameters[parameter_num] = Some(Param::Char {
                    value: ESCAPE_SEQUENCE_VALUES[i],
                    text: TRANSLATED_ESCAPE_SEQUENCES[i].as_bytes().to_vec(),
                    // gas has no `\v` escape and reads 'v'
                    verbatim: ESCAPE_SEQUENCES[i] != "\\v",
                });
                print_debug(ll, "\t\tParameter is an escape sequence and has been translated", &[]);
                check_not_pointer(cmd, parameter_num, input_file_name, state, "a character cannot be a pointer");
                continue 'params;
            } else if parameter.len() == 1 {
                let c = parameter[0];
                // gas reads `'\'` as an escaped quote
                cmd.parameters[parameter_num] = Some(Param::Char { value: c, text: vec![b'\'', c, b'\''], verbatim: c != b'\\' });
                print_debug(ll, "\t\tParameter is a character, translated to: %s", &[Arg::S(&cmd.param_text(parameter_num, target))]);
                check_not_pointer(cmd, parameter_num, input_file_name, state, "a character cannot be a pointer");
                continue 'params;
            }
            print_debug(ll, "\t\tParameter is neither a character nor an escape sequence", &[]);

            // Now check if it is an ASCII-code
            if let Some(r) = parse_decimal(&parameter).filter(|r| (0..=255).contains(&r.value)) {
                print_debug(ll, "\t\tParameter is an ASCII-code", &[]);
                check_not_pointer(cmd, parameter_num, input_file_name, state, "a character cannot be a pointer");
                // gas reads numbers with leading zeros as octal
                let digits = parameter.strip_prefix(b"-").or(parameter.strip_prefix(b"+")).unwrap_or(&parameter);
                let verbatim = !(digits.len() > 1 && digits[0] == b'0');
                cmd.parameters[parameter_num] = Some(Param::Char { value: r.value as u8, text: parameter, verbatim });
                continue 'params;
            }
            print_debug(ll, "\t\tParameter is not an ASCII-code", &[]);
        }

        if allowed_types & PARAM_MONKE_LABEL != 0 {
            let mut a_used = false;
            let mut u_used = false;
            let mut unexpected_character = false;
            for &c in &parameter {
                match c {
                    b'a' => a_used = true,
                    b'u' => u_used = true,
                    _ => {
                        unexpected_character = true;
                        break;
                    }
                }
            }
            if a_used && u_used && !unexpected_character {
                print_debug(ll, "\t\tParameter is a valid Monke jump label", &[]);
                check_not_pointer(cmd, parameter_num, input_file_name, state, "a jump marker cannot be a pointer");
                cmd.parameters[parameter_num] = Some(Param::MonkeLabel(parameter));
                continue 'params;
            }
            print_debug(ll, "\t\tParameter is not a valid Monke jump label", &[]);
        }

        if allowed_types & PARAM_FUNC_NAME != 0 {
            let mut unexpected_character = false;
            for (i, &c) in parameter.iter().enumerate() {
                if i == 0 && c.is_ascii_digit() {
                    unexpected_character = true;
                    print_debug(ll, "\t\tParameter is not a valid function name, as there is a number in the first position", &[]);
                    break;
                }
                if !(c == b'_' || c == b'$' || c == b'.' || c.is_ascii_alphanumeric()) {
                    unexpected_character = true;
                    print_debug(
                        ll,
                        "\t\tParameter is not a valid function name, unexpected character found at position %lu",
                        &[Arg::U(i as u64)],
                    );
                    break;
                }
            }
            if !unexpected_character {
                print_debug(ll, "\t\tParameter is a valid function name", &[]);
                check_not_pointer(cmd, parameter_num, input_file_name, state, "a function name cannot be a pointer");
                // (validated with the macOS `_` prefix, which is added again when generating code)
                let func = match &cmd.parameters[parameter_num] {
                    Some(Param::Raw { text, mangle }) => Param::FuncName { name: cstr(text).to_vec(), mangle: *mangle },
                    _ => Param::FuncName { name: parameter, mangle: false },
                };
                cmd.parameters[parameter_num] = Some(func);
                continue 'params;
            }
        }

        print_debug(ll, "No checks succeeded, invalid parameter!", &[]);
        if state.compile_mode != CompileMode::Bully {
            print_error(
                state,
                input_file_name,
                cmd.line_num as u64,
                "invalid parameter provided: \"%s\"",
                1,
                &[Arg::S(&parameter)],
            );
            if state.compile_mode != CompileMode::Obfuscated {
                print_parameter_usage_note(allowed_types);
            }
        } else {
            // Replace the parameter with something pseudo-random that works, derived from the
            // global "computedIndex".
            for &c in &parameter {
                state.computed_index = state.computed_index.wrapping_mul(u64::from(c)) / 2;
            }

            // Choose a parameter. If it is not allowed, rotate the bitmask until we find a valid one
            let mut chosen: u8 = 1 << (state.computed_index % 8);
            while allowed_types & chosen == 0 {
                chosen = if chosen == 1 << 7 { 1 } else { chosen << 1 };
            }

            let new_param = match chosen {
                PARAM_REG64 | PARAM_REG32 | PARAM_REG16 | PARAM_REG8 => {
                    get_random_register(chosen, state.computed_index)
                }
                PARAM_DECIMAL => Some(decimal(((state.computed_index as u32) % 128).to_string().into_bytes())),
                PARAM_CHAR => {
                    let value = (state.computed_index % 128) as u8;
                    Some(Param::Char { value, text: value.to_string().into_bytes(), verbatim: true })
                }
                PARAM_MONKE_LABEL => {
                    let length = (state.computed_index % 7 + 2) as usize;
                    let mut p = Vec::with_capacity(length);
                    for _ in 0..length {
                        p.push(if state.computed_index.is_multiple_of(2) { b'u' } else { b'a' });
                        state.computed_index = state.computed_index.wrapping_mul(3) / 2;
                    }
                    Some(Param::MonkeLabel(p))
                }
                // (without the macOS `_` prefix)
                PARAM_FUNC_NAME => Some(Param::FuncName {
                    name: FUNCTION_NAMES[(state.computed_index % FUNCTION_NAMES.len() as u64) as usize]
                        .as_bytes()
                        .to_vec(),
                    mangle: false,
                }),
                _ => {
                    print_internal_compiler_error(
                        "Random parameter generation unsupported for paramType %u",
                        true,
                        &[Arg::U(chosen as u64)],
                    );
                    std::process::exit(1);
                }
            };
            if cmd.is_pointer as usize == parameter_num + 1 {
                cmd.is_pointer = 0;
            }
            cmd.parameters[parameter_num] = new_param;
        }
    }

    let bully = state.compile_mode == CompileMode::Bully;
    if bully && cmd.is_pointer != 0 {
        // Well then it's not a pointer :shrug: (before check 3, so that a former pointer gets
        // the size of the other register)
        if pointer_problem(cmd).is_some() {
            cmd.is_pointer = 0;
        }
    }

    // We only compare parameters if there are at least two of them
    if used_parameters >= 2 {
        // 1: If a number and a register are used, the number must fit in the register
        // 2: If a number and a 64 Bit register are used (and the command is not a mov-command),
        //    the number must be sign-extendable from 32 Bits
        let (t0, t1) = (cmd.param_type(0), cmd.param_type(1));
        if (t0 == PARAM_DECIMAL && param_is_reg(t1)) || (t1 == PARAM_DECIMAL && param_is_reg(t0)) {
            let decimal_index = if t0 == PARAM_DECIMAL { 0 } else { 1 };
            let reg_index = 1 - decimal_index;
            let reg_size = get_register_size(cmd.param_type(reg_index));

            let number = decimal_param_value(cmd, decimal_index);
            // Two's complement: positive numbers need 64 - clz bits, negative ones 64 - clz(~n) + 1.
            // (__builtin_clzll(0) for n = -1 is undefined in C; this uses 64 like ARM's CLZ.)
            let bits_needed: u32 = if number == 0 {
                1
            } else if number.leading_zeros() > 0 {
                64 - number.leading_zeros()
            } else {
                64 - (!number).leading_zeros() + 1
            };
            if bits_needed > reg_size {
                if state.compile_mode != CompileMode::Bully {
                    let dec = cmd.param_text(decimal_index, target).into_owned();
                    let reg = cmd.param_text(reg_index, target).into_owned();
                    print_error(
                        state,
                        input_file_name,
                        cmd.line_num as u64,
                        "invalid parameter combination: '%s' (%u bits) does not fit into register '%s' of size %u",
                        3,
                        &[Arg::S(&dec), Arg::U(bits_needed as u64), Arg::S(&reg), Arg::U(reg_size as u64)],
                    );
                } else {
                    // We replace the number with something that is guaranteed to fit into all registers
                    cmd.parameters[decimal_index] =
                        Some(decimal(((state.computed_index as u32) % 256).to_string().into_bytes()));
                }
            } else if command.command_type != COMMAND_TYPE_MOV
                && reg_size == 64
                && !((number as u64 & 0xFFFF_FFFF_8000_0000) == 0 || (number | 0x7FFF_FFFF) == -1)
            {
                if state.compile_mode != CompileMode::Bully {
                    print_error(
                        state,
                        input_file_name,
                        cmd.line_num as u64,
                        "invalid parameter combination: 64 Bit arithmetic operation commands require the decimal number to be sign-extendable from 32 Bits",
                        0,
                        &[],
                    );
                } else {
                    // We just make the number shorter than 32 bits :bigBrain:
                    shorten_decimal(&mut cmd.parameters[decimal_index], (state.computed_index % 32) as usize);
                    // Also, change computedIndex. Just because
                    state.computed_index = state.computed_index.wrapping_add((number & 0xFFF) as u64);
                }
            }
        }

        // 3: If two registers are used, they must be of the same size
        let mut current_reg = 0u8;
        for i in 0..used_parameters {
            let param_type = cmd.param_type(i);
            let not_pointer = cmd.is_pointer as usize != i + 1;
            if current_reg == 0 {
                if param_is_reg(param_type) && not_pointer {
                    current_reg = param_type;
                }
            } else if param_is_reg(param_type) && not_pointer && param_type != current_reg {
                if state.compile_mode != CompileMode::Bully {
                    print_error(
                        state,
                        input_file_name,
                        cmd.line_num as u64,
                        "invalid parameter combination: cannot combine registers of different size",
                        0,
                        &[],
                    );
                } else {
                    cmd.parameters[i] = get_random_register(current_reg, state.computed_index);
                }
            }
        }

        // ah/bh/ch/dh can't be encoded together with a register that needs a REX prefix: use
        // another 8 bit register instead
        if bully {
            if let Some((high, _)) = high_byte_conflict(cmd) {
                cmd.parameters[high] =
                    Some(register(NON_HIGH_8_BIT[(state.computed_index % NON_HIGH_8_BIT.len() as u64) as usize]));
            }
        }

        // 4: If there's a pointer parameter, the other parameter must be a register, otherwise the
        // operand size is unknown. Skipped in bully mode (handled by the translator).
        if state.compile_mode != CompileMode::Bully
            && cmd.is_pointer != 0
            && !param_is_reg(cmd.param_type((cmd.is_pointer % 2) as usize))
        {
            print_error(
                state,
                input_file_name,
                cmd.line_num as u64,
                "invalid parameter combination: operand size unknown",
                0,
                &[],
            );
        }
    }

    // `push` only takes sign-extended 32-bit immediates
    let push_too_large = cmd.opcode == Opcode::Push
        && cmd.param_type(0) == PARAM_DECIMAL
        && !fits_sign_extended_32(decimal_param_value(cmd, 0));
    if bully {
        if push_too_large {
            // Shorten it like the "sign-extendable from 32 Bits" fix-up above
            shorten_decimal(&mut cmd.parameters[0], (state.computed_index % 32) as usize);
        }
        return;
    }

    // Combinations the C compiler accepted, but that can't be assembled. Only reported if the
    // checks above found nothing wrong with this command.
    if state.compiler_errors != errors_before {
        return;
    }
    let line = cmd.line_num as u64;
    if let Some(problem) = pointer_problem(cmd) {
        let message = match problem {
            PointerProblem::Base => "invalid parameter combination: a pointer must be a 64 or 32 bit register",
            PointerProblem::Unsupported => "invalid parameter combination: this command cannot use a pointer here",
        };
        print_error(state, input_file_name, line, message, 0, &[]);
    } else if cmd.is_pointer != 0
        && pointer_use(cmd.opcode, cmd.is_pointer as usize - 1) == PointerUse::NeedsSize
    {
        print_error(state, input_file_name, line, "invalid parameter combination: operand size unknown", 0, &[]);
    } else if let Some((high, other)) = high_byte_conflict(cmd) {
        let (high, other) = (cmd.param_text(high, target).into_owned(), cmd.param_text(other, target).into_owned());
        print_error(
            state,
            input_file_name,
            line,
            "invalid parameter combination: '%s' cannot be combined with '%s'",
            2,
            &[Arg::S(&high), Arg::S(&other)],
        );
    } else if push_too_large {
        print_error(
            state,
            input_file_name,
            line,
            "invalid parameter combination: pushed decimal numbers must be sign-extendable from 32 Bits",
            0,
            &[],
        );
    }
}

/// 8 bit registers without ah, bh, ch and dh.
const NON_HIGH_8_BIT: [&str; 16] = [
    "al", "bl", "cl", "dl", "dil", "sil", "spl", "bpl", "r8b", "r9b", "r10b", "r11b", "r12b",
    "r13b", "r14b", "r15b",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PointerProblem {
    /// `[al]`, `[ax]`: only 64 and 32 bit registers can address memory
    Base,
    /// The translation pattern can't use a memory operand there (see [`PointerUse`])
    Unsupported,
}

fn pointer_problem(cmd: &ParsedCommand) -> Option<PointerProblem> {
    let index = (cmd.is_pointer as usize).checked_sub(1)?;
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
    if cmd.opcode.command().used_parameters != 2 {
        return None;
    }
    let high = (0..2).find(|&i| {
        cmd.is_pointer as usize != i + 1 && matches!(cmd.parameters[i], Some(Param::Reg(Reg::R8H(_))))
    })?;
    let other = 1 - high;
    matches!(cmd.parameters[other], Some(Param::Reg(r)) if needs_rex(r)).then_some((high, other))
}

/// Whether a 64 bit immediate can be encoded as a sign-extended 32 bit immediate.
fn fits_sign_extended_32(number: i64) -> bool {
    i32::try_from(number).is_ok()
}
