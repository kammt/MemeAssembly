//! Port of compiler/translator/translator.c

use super::analyser::main_function_exists;
use super::commands::*;
use super::libc::{asctime_now, strtol, strtoll};
use super::log::{exit, print_debug, print_internal_compiler_error, Arg, VERSION_STRING};
use crate::target::Target;

fn martyrdom_code(target: Target) -> String {
    let mut s = String::from(
        "push rax\n    push rdi\n    push rsi\n    push rdx\n    push r10\n    push rcx\n    push r11\n    \n    lea rax, [rip + killParent]\n    mov [rip + .Lsa_handler], rax\n",
    );
    if target == Target::MacOS {
        // Signaling SIGINT on MacOS segfaults unless the second qword of the sigaction struct
        // contains the address as well
        s.push_str("    mov [rip + .Lsa_handler_2], rax\n");
    }
    s.push('\n');
    if target == Target::Linux {
        s.push_str("    mov rax, 13\n");
    } else {
        s.push_str("    mov rax, 0x200002E\n");
    }
    s.push_str(
        "    mov rdi, 2\n    lea rsi, [rip + .LsigStruct]\n    xor rdx, rdx\n    mov r10, 8\n    syscall\n    \n    pop r11\n    pop rcx\n    pop r10\n    pop rdx\n    pop rsi\n    pop rdi\n    pop rax\n\n",
    );
    s
}

struct Writer {
    buf: Vec<u8>,
}

impl Writer {
    fn s(&mut self, s: &str) {
        self.buf.extend_from_slice(s.as_bytes());
    }
    fn b(&mut self, b: &[u8]) {
        self.buf.extend_from_slice(b);
    }
}

/// The current working directory as `getcwd()` returns it (`(null)` on failure, like glibc's
/// printf of a NULL string).
pub(super) fn getcwd_bytes() -> Vec<u8> {
    match std::env::current_dir() {
        #[cfg(unix)]
        Ok(p) => {
            use std::os::unix::ffi::OsStrExt;
            p.as_os_str().as_bytes().to_vec()
        }
        #[cfg(not(unix))]
        Ok(p) => p.to_string_lossy().into_owned().into_bytes(),
        Err(_) => b"(null)".to_vec(),
    }
}

/// The absolute path of an input file as written into the debug information (the C code
/// prefixed relative names with `getcwd()` for its STABS `N_SO` entry).
pub(super) fn debug_file_path(input_file: &[u8]) -> Vec<u8> {
    let mut path = Vec::new();
    if input_file.first() != Some(&b'/') {
        path.extend_from_slice(&getcwd_bytes());
        path.push(b'/');
    }
    path.extend_from_slice(input_file);
    path
}

/// `.file N "path"` for gas' DWARF line tables (replaces the C compiler's STABS `N_SO`).
fn write_debug_file(w: &mut Writer, file_number: usize, input_file: &[u8]) {
    w.s(&format!(".file {file_number} \""));
    for &c in &debug_file_path(input_file) {
        if c == b'"' || c == b'\\' {
            w.buf.push(b'\\');
        }
        w.buf.push(c);
    }
    w.s("\"\n");
}

/// Operand sizes a bully-mode pointer can get, in bytes (index = `computedIndex % 4`).
pub(super) const BULLY_POINTER_SIZES: [u32; 4] = [1, 2, 4, 8];

/// In bully mode, the "operand size unknown" checks of the analyser are skipped and the operand
/// size of a pointer is chosen here (in bytes), if it is needed.
pub(super) fn bully_pointer_size(state: &CompileState, cmd: &ParsedCommand) -> Option<u32> {
    let index = (cmd.is_pointer as usize).checked_sub(1)?;
    (state.compile_mode == CompileMode::Bully && cmd.bully_pointer_needs_size(index))
        .then(|| BULLY_POINTER_SIZES[(state.computed_index % 4) as usize])
}

/// `it's over 9000` compares with the immediate 9000, which an 8-bit operand can never exceed
/// (gas silently truncated it to 0x28): the command then always halts.
pub(super) fn over_9000_always_halts(state: &CompileState, cmd: &ParsedCommand) -> bool {
    cmd.opcode == OP_OVER_9000
        && match bully_pointer_size(state, cmd) {
            Some(size) => size == 1,
            None => cmd.is_pointer == 0 && cmd.param_types[0] == PARAM_REG8,
        }
}

/// Bully mode: an immediate stored to a pointer whose size was chosen randomly is truncated to
/// that size (a QWORD immediate must be a sign-extended 32-bit value).
fn truncate_to_operand_size(value: i64, size: u32) -> i64 {
    match size {
        1 if !(-128..=255).contains(&value) => value & 0xFF,
        2 if !(-32768..=65535).contains(&value) => value & 0xFFFF,
        4 if !(i32::MIN as i64..=u32::MAX as i64).contains(&value) => value & 0xFFFF_FFFF,
        8 => value as i32 as i64,
        _ => value,
    }
}

/// Whether gas reads the identifier `name` as something other than a symbol when it is used as
/// a `call` operand (`$` and `.` are the current address, register names make the call
/// indirect, `byte`/`near`/... are constants, `and`/`not`/... operators). Mirrors the
/// emitter's identifier classification.
pub(super) fn gas_misreads_symbol(name: &[u8]) -> bool {
    let l = String::from_utf8_lossy(name).to_ascii_lowercase();
    if l == "$" || l == "." {
        return true;
    }
    let numbered = |prefix: &str, max: u32| -> bool {
        l.strip_prefix(prefix).is_some_and(|n| {
            !n.is_empty()
                && (n.len() == 1 || !n.starts_with('0'))
                && n.chars().all(|c| c.is_ascii_digit())
                && n.parse::<u32>().is_ok_and(|v| v <= max)
        })
    };
    // General purpose registers, including r8-r31 with their b/w/d forms
    const LEGACY: [&str; 36] = [
        "rax", "rcx", "rdx", "rbx", "rsp", "rbp", "rsi", "rdi", "eax", "ecx", "edx", "ebx", "esp",
        "ebp", "esi", "edi", "ax", "cx", "dx", "bx", "sp", "bp", "si", "di", "al", "cl", "dl", "bl",
        "spl", "bpl", "sil", "dil", "ah", "ch", "dh", "bh",
    ];
    if LEGACY.contains(&l.as_str()) {
        return true;
    }
    if let Some(rest) = l.strip_prefix('r') {
        let rest = rest.strip_suffix(['b', 'w', 'd']).unwrap_or(rest);
        if !rest.is_empty()
            && (rest.len() == 1 || !rest.starts_with('0'))
            && rest.chars().all(|c| c.is_ascii_digit())
            && rest.parse::<u32>().is_ok_and(|v| (8..=31).contains(&v))
        {
            return true;
        }
    }
    if numbered("xmm", 31)
        || numbered("ymm", 31)
        || numbered("zmm", 31)
        || numbered("tmm", 7)
        || numbered("mm", 7)
        || numbered("k", 7)
        || numbered("cr", 15)
        || numbered("dr", 15)
    {
        return true;
    }
    matches!(
        l.as_str(),
        "st" | "es" | "cs" | "ss" | "ds" | "fs" | "gs" | "rip" | "eip" | "flat"
            | "and" | "or" | "xor" | "mod" | "shl" | "shr" | "eq" | "ne" | "lt" | "le" | "gt" | "ge"
            | "not"
            | "byte" | "word" | "dword" | "fword" | "qword" | "mmword" | "tbyte" | "oword"
            | "xmmword" | "ymmword" | "zmmword" | "offset" | "short" | "near" | "far"
    )
}

/// The value of a character parameter, if gas would read its C text differently: `'\v'` (gas
/// has no `\v` escape and reads 'v'), `'\'` (the single character backslash; gas reads the
/// escaped quote) and ASCII codes with leading zeros (validated as decimal by `strtol(.., 10)`,
/// but octal to gas).
fn char_fixup(parameter: &[u8]) -> Option<i64> {
    match parameter {
        b"'\\v'" => Some(11),
        b"'\\'" => Some(b'\\' as i64),
        [b'\'', ..] => None,
        _ => {
            let digits = parameter.strip_prefix(b"-").or(parameter.strip_prefix(b"+")).unwrap_or(parameter);
            (digits.len() > 1 && digits[0] == b'0').then(|| strtol(parameter).value)
        }
    }
}

/// The text substituted for `{index}` in a translation pattern.
pub(super) fn param_text(state: &CompileState, cmd: &ParsedCommand, index: usize) -> Vec<u8> {
    let parameter = cmd.param(index);
    let mut text = Vec::new();
    if cmd.is_pointer as usize == index + 1 {
        if let Some(size) = bully_pointer_size(state, cmd) {
            const OPERAND_SIZES: [&str; 4] = ["BYTE PTR", "WORD PTR", "DWORD PTR", "QWORD PTR"];
            let i = BULLY_POINTER_SIZES.iter().position(|s| *s == size).unwrap_or(3);
            text.extend_from_slice(OPERAND_SIZES[i].as_bytes());
            text.extend_from_slice(b" [");
        } else {
            text.push(b'[');
        }
        text.extend_from_slice(parameter);
        text.push(b']');
    } else if cmd.param_types[index] == PARAM_DECIMAL {
        let mut value = strtoll(parameter).value;
        if let Some(size) = bully_pointer_size(state, cmd) {
            value = truncate_to_operand_size(value, size);
        }
        // Decimal numbers are written as hex (issue #73)
        text.extend_from_slice(format!("0x{:X}", value as u64).as_bytes());
    } else if let Some(value) = (cmd.param_types[index] == PARAM_CHAR).then(|| char_fixup(parameter)).flatten() {
        text.extend_from_slice(value.to_string().as_bytes());
    } else if cmd.opcode == OP_CALL && gas_misreads_symbol(parameter) {
        text.push(b'"');
        text.extend_from_slice(parameter);
        text.push(b'"');
    } else {
        text.extend_from_slice(parameter);
    }
    text
}

/// Whether no command produces any code after bully mode's injected main (no functions at all,
/// everything deleted, or `-O69420`).
pub(super) fn bully_main_is_empty(state: &CompileState) -> bool {
    !(0..state.files.len()).any(|file| {
        (0..state.files[file].functions.len()).any(|func| {
            state.function_commands(file, func).iter().any(|cmd| {
                cmd.translate
                    && (state.optimisation_level != OPT_69420
                        || COMMAND_LIST[cmd.opcode as usize].command_type == COMMAND_TYPE_FUNC_DEF)
            })
        })
    })
}

fn translate_to_assembly(
    state: &CompileState,
    w: &mut Writer,
    cmd: &ParsedCommand,
    file_num: usize,
) {
    let command = &COMMAND_LIST[cmd.opcode as usize];
    let is_def = command.command_type == COMMAND_TYPE_FUNC_DEF;
    if !is_def && state.optimisation_level == OPT_69420 {
        print_debug(state.log_level, "\tCommand is not a function declaration, abort.", &[]);
        return;
    }

    // Function definitions get a row too, so the martyrdom prologue that follows `main:` and
    // `break <function>` are attributed to the definition line.
    if state.use_debug_info {
        w.s(&format!("\t.loc {} {}\n", file_num + 1, cmd.line_num));
    }

    if !is_def {
        w.s("\t");
    }
    let pattern = if over_9000_always_halts(state, cmd) {
        "hlt".as_bytes()
    } else {
        command.translation_pattern.as_bytes()
    };
    let mut i = 0;
    while i < pattern.len() {
        if pattern[i] == b'{' && pattern.get(i + 2) == Some(&b'}') {
            let format_specifier = pattern[i + 1];
            if format_specifier == b'F' {
                w.s(&file_num.to_string());
            } else if format_specifier >= b'0' && format_specifier < command.used_parameters + b'0' {
                let index = (format_specifier - b'0') as usize;
                w.b(&param_text(state, cmd, index));
            } else {
                print_internal_compiler_error(
                    "Invalid translation format specifier '%c' for opcode %u",
                    true,
                    &[Arg::C(format_specifier), Arg::U(cmd.opcode as u64)],
                );
                exit(1);
            }
            i += 3;
        } else {
            w.buf.push(pattern[i]);
            i += 1;
        }
    }
    w.s("\n");

    match state.optimisation_level {
        OPT_1 => w.s("\tnop\n"),
        OPT_2 => w.s("\tpush rax\n\tpop rax\n"),
        OPT_3 => w.s("\tmovups [rsp + 8], xmm0\n\tmovups xmm0, [rsp + 8]\n"),
        // If we get here, this was a function declaration
        OPT_69420 => w.s("\txor rax, rax\n\tret\n"),
        _ => {}
    }
}

/// Generates the complete assembly text (what `-S` writes).
pub fn write_to_string(state: &CompileState) -> Vec<u8> {
    let target = state.target;
    let mut w = Writer { buf: Vec::with_capacity(16 * 1024) };

    w.s(&format!("#\n# Generated by the MemeAssembly compiler {} on {}#\n", VERSION_STRING, asctime_now()));
    w.s(".intel_syntax noprefix\n");

    // Define all functions as global
    for file in 0..state.files.len() {
        for func in 0..state.files[file].functions.len() {
            let def = &state.function_commands(file, func)[0];
            if def.translate {
                w.s(".global ");
                w.b(def.param(0));
                w.s("\n");
            }
        }
    }

    if target == Target::Windows {
        // To interact with the Windows API, we need to reference the needed functions
        w.s("\n.extern GetStdHandle\n.extern WriteFile\n.extern ReadFile\n");
    }

    w.s("\n.data\n\t");
    w.s(".LCharacter: .ascii \"a\"\n\t.Ltmp64: .byte 0, 0, 0, 0, 0, 0, 0, 0\n");

    // Struct for martyrdom command
    match target {
        Target::Linux => w.s("\t.LsigStruct:\n\t\t.Lsa_handler: .quad 0\n\t\t.quad 0x04000000\n\t\t.quad 0, 0\n\n"),
        Target::MacOS => w.s("\t.LsigStruct:\n\t\t.Lsa_handler: .quad 0\n\t\t.Lsa_handler_2: .quad 0\n\t\t.quad 0, 0\n\n"),
        Target::Windows => {}
    }

    w.s("\n\n.text\n\t");
    w.s("\n\n.Ltext0:\n");

    if target != Target::Windows {
        let linux = target == Target::Linux;
        w.s("killParent:\n");
        w.s(if linux { "    mov rax, 110\n" } else { "    mov rax, 0x2000027\n" });
        w.s("    syscall\n\n    mov rdi, rax\n    mov rsi, 9\n");
        w.s(if linux { "    mov rax, 62\n" } else { "    mov rax, 0x2000025\n" });
        w.s("    syscall\n\n    mov rdi, 0\n");
        // exit(0). The C code used the Linux syscall number 60 on macOS as well.
        w.s(if linux { "    mov rax, 60\n" } else { "    mov rax, 0x2000001\n" });
        w.s("    syscall\n    ret\n\n");
    }

    // In bully mode, a missing main function is "fixed" by making the code at the start main
    if state.compile_mode == CompileMode::Bully
        && state.output_mode == OutputMode::Executable
        && !main_function_exists(state)
    {
        // The C code always named it `main` (macOS needs `_main`) and also inserted the
        // martyrdom code on Windows, where killParent/.LsigStruct do not exist.
        let main_name = state.main_function_name();
        w.s("\n.global ");
        w.b(main_name);
        w.s("\n\t\n");
        w.b(main_name);
        w.s(":\n\t");
        if target != Target::Windows {
            w.s(&martyrdom_code(target));
        }
        // Without any code to fall into, the C version ran into writechar (or off the end of
        // the text section) and returned garbage: return 0 instead.
        if bully_main_is_empty(state) {
            w.s("xor rax, rax\n\tret\n");
        }
    }

    let main_name = state.main_function_name();
    for (i, file) in state.files.iter().enumerate() {
        if state.use_debug_info {
            write_debug_file(&mut w, i + 1, &file.file_name);
        }

        let mut line = 0usize;
        for func in 0..file.functions.len() {
            let commands = state.function_commands(i, func);
            let function_name = commands[0].param(0);

            for (k, current_command) in commands.iter().enumerate() {
                if target != Target::Windows && state.martyrdom && k == 1 && function_name == main_name {
                    w.s(&martyrdom_code(target));
                }

                // Print the confused stonks label now if it should be at this position
                if line == file.random_index {
                    w.s(&format!("\t.LConfusedStonks_{i}: \n"));
                }

                if current_command.translate {
                    translate_to_assembly(state, &mut w, current_command, i);
                }
                line += 1;
            }
        }
    }

    // With -O69420 all commands are optimised out, so these are not needed
    if state.optimisation_level != OPT_69420 {
        // The helpers are not source code: line 0 instead of the last command's line
        if state.use_debug_info && !state.files.is_empty() {
            w.s(&format!("\t.loc {} 0\n", state.files.len()));
        }
        if target == Target::Windows {
            // All caller-saved registers are preserved. 48 bytes = shadow space + the 5th
            // argument's slot, keeping the stack 16-byte aligned (the C code reserved only the
            // 32 bytes of shadow space, so the 5th argument overwrote the saved r9).
            for (name, handle, api) in [("writechar", -11, "WriteFile"), ("readchar", -10, "ReadFile")] {
                w.s(&format!(
                    "\n\n{name}:\n\tpush rcx\n\tpush rax\n\tpush rdx\n\tpush r8\n\tpush r9\n\tpush r10\n\tpush r11\n\tsub rsp, 48\n\tmov rcx, {handle}\n\tcall GetStdHandle\n\tmov rcx, rax\n\tlea rdx, [rip + .LCharacter]\n\tmov r8, 1\n\tlea r9, [rip + .Ltmp64]\n\tmov QWORD PTR [rsp + 32], 0\n\tcall {api}\n\tadd rsp, 48\n\tpop r11\n\tpop r10\n\tpop r9\n\tpop r8\n\tpop rdx\n\tpop rax\n\tpop rcx\n\tret\n"
                ));
            }
        } else {
            let linux = target == Target::Linux;
            w.s("\n\nwritechar:\n\tpush rcx\n\tpush r11\n\tpush rax\n\tpush rdi\n\tpush rsi\n\tpush rdx\n\tmov rdx, 1\n\tlea rsi, [rip + .LCharacter]\n\tmov rdi, 1\n\t");
            w.s(if linux { "mov rax, 1\n\t" } else { "mov rax, 0x2000004\n\t" });
            w.s("syscall\n\tpop rdx\n\tpop rsi\n\tpop rdi\n\tpop rax\n\tpop r11\n\tpop rcx\n\t\n\tret\n");

            w.s("\n\nreadchar:\n\tpush rcx\n\tpush r11\n\tpush rax\n\tpush rdi\n\tpush rsi\n\tpush rdx\n\n\tmov rdx, 1\n\tlea rsi, [rip + .LCharacter]\n\tmov rdi, 0\n\t");
            w.s(if linux { "mov rax, 0\n\t" } else { "mov rax, 0x2000003\n\t" });
            w.s("syscall\n\n\tpop rdx\n\tpop rsi\n\tpop rdi\n\tpop rax\n\tpop r11\n\tpop rcx\n\tret\n");
        }
    }

    if state.optimisation_level == OPT_S {
        w.s(".align 536870912\n");
    }

    w.buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_values_gas_would_misread() {
        assert_eq!(char_fixup(b"'\\v'"), Some(11));
        assert_eq!(char_fixup(b"'\\'"), Some(92));
        assert_eq!(char_fixup(b"'\\n'"), None);
        assert_eq!(char_fixup(b"'a'"), None);
        assert_eq!(char_fixup(b"010"), Some(10));
        assert_eq!(char_fixup(b"-010"), Some(-10));
        assert_eq!(char_fixup(b"0065"), Some(65));
        assert_eq!(char_fixup(b"0"), None);
        assert_eq!(char_fixup(b"65"), None);
    }

    #[test]
    fn misread_call_symbols() {
        for name in ["$", ".", "rax", "RAX", "r8", "r15d", "r31b", "xmm0", "ah", "and", "byte", "QWORD", "st", "fs", "rip", "flat", "near"] {
            assert!(gas_misreads_symbol(name.as_bytes()), "{name}");
        }
        for name in ["main", "foo", "r32", "r08", "r", "xmm32", "rax1", "st0", "_and", "bytes", "foo$bar"] {
            assert!(!gas_misreads_symbol(name.as_bytes()), "{name}");
        }
    }

    #[test]
    fn immediate_truncation() {
        assert_eq!(truncate_to_operand_size(255, 1), 255);
        assert_eq!(truncate_to_operand_size(-128, 1), -128);
        assert_eq!(truncate_to_operand_size(300, 1), 44);
        assert_eq!(truncate_to_operand_size(65536, 2), 0);
        assert_eq!(truncate_to_operand_size(-1, 4), -1);
        assert_eq!(truncate_to_operand_size(4294967296, 4), 0);
        assert_eq!(truncate_to_operand_size(2147483648, 8), i32::MIN as i64);
        assert_eq!(truncate_to_operand_size(-5, 8), -5);
    }
}
