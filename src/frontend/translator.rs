//! Port of compiler/translator/translator.c

use super::analyser::main_function_exists;
use super::commands::*;
use super::libc::{asctime_now, strtoll};
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

/// The text substituted for `{index}` in a translation pattern.
pub(super) fn param_text(state: &CompileState, cmd: &ParsedCommand, index: usize) -> Vec<u8> {
    let command = &COMMAND_LIST[cmd.opcode as usize];
    let parameter = cmd.param(index);
    let mut text = Vec::new();
    if cmd.is_pointer as usize == index + 1 {
        // In bully mode, the "operand size unknown" check of parameters.c is skipped, so the
        // operand size is chosen here.
        if state.compile_mode == CompileMode::Bully
            && command.used_parameters == 2
            && cmd.bully_pointer_needs_size(index)
        {
            const OPERAND_SIZES: [&str; 4] = ["BYTE PTR", "WORD PTR", "DWORD PTR", "QWORD PTR"];
            text.extend_from_slice(OPERAND_SIZES[(state.computed_index % 4) as usize].as_bytes());
            text.extend_from_slice(b" [");
        } else {
            text.push(b'[');
        }
        text.extend_from_slice(parameter);
        text.push(b']');
    } else if cmd.param_types[index] == PARAM_DECIMAL {
        // Decimal numbers are written as hex (issue #73)
        text.extend_from_slice(format!("0x{:X}", strtoll(parameter).value as u64).as_bytes());
    } else {
        text.extend_from_slice(parameter);
    }
    text
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

    if state.use_debug_info && !is_def {
        w.s(&format!("\t.loc {} {}\n", file_num + 1, cmd.line_num));
    }

    if !is_def {
        w.s("\t");
    }
    let pattern = command.translation_pattern.as_bytes();
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
        if target == Target::Windows {
            w.s("\n\nwritechar:\n\tpush rcx\n\tpush rax\n\tpush rdx\n\tpush r8\n\tpush r9\n\tsub rsp, 32\n\tmov rcx, -11\n\tcall GetStdHandle\n\tmov rcx, rax\n\tlea rdx, [rip + .LCharacter]\n\tmov r8, 1\n\tlea r9, [rip + .Ltmp64]\n\tmov QWORD PTR [rsp + 32], 0\n\tcall WriteFile\n\tadd rsp, 32\n\tpop r9\n\tpop r8\n\tpop rdx\n\tpop rax\n\tpop rcx\n\tret\n");
            w.s("\n\nreadchar:\n\tpush rcx\n\tpush rax\n\tpush rdx\n\tpush r8\n\tpush r9\n\tsub rsp, 32\n\tmov rcx, -10\n\tcall GetStdHandle\n\tmov rcx, rax\n\tlea rdx, [rip + .LCharacter]\n\tmov r8, 1\n\tlea r9, [rip + .Ltmp64]\n\tmov QWORD PTR [rsp + 32], 0\n\tcall ReadFile\n\tadd rsp, 32\n\tpop r9\n\tpop r8\n\tpop rdx\n\tpop rax\n\tpop rcx\n\tret\n");
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
