//! Port of compiler/logger/log.c.

use std::io::{ErrorKind, Write};

use super::commands::{CompileMode, CompileState, LogLevel};
use crate::target::Target;

pub const RED: &str = "\x1B[31m";
pub const GRN: &str = "\x1B[32m";
pub const YEL: &str = "\x1B[33m";
pub const BLU: &str = "\x1B[34m";
pub const MAG: &str = "\x1B[35m";
pub const RESET: &str = "\x1B[0m";

pub const VERSION_STRING: &str = "v1.6";

/// Exit code when stdout is a closed pipe (what shells report for a process killed by SIGPIPE).
pub const BROKEN_PIPE_EXIT_CODE: i32 = 141;

/// Writes raw bytes to stdout. If the reader of a pipe has gone away, the compiler ends quietly
/// with [`BROKEN_PIPE_EXIT_CODE`]; any other write error (e.g. `/dev/full`) is reported and
/// ends the program with exit code 1.
pub fn out(bytes: &[u8]) {
    let mut so = std::io::stdout().lock();
    let result = so.write_all(bytes).and_then(|()| so.flush());
    drop(so);
    if let Err(e) = result {
        if e.kind() == ErrorKind::BrokenPipe {
            std::process::exit(BROKEN_PIPE_EXIT_CODE);
        }
        err_str(&format!("Failed to write to stdout: {}\n", super::libc::io_error_message(&e)));
        std::process::exit(1);
    }
}

pub fn out_str(s: &str) {
    out(s.as_bytes());
}

/// Writes raw bytes to stderr (write errors are ignored).
pub fn err(bytes: &[u8]) {
    let mut se = std::io::stderr().lock();
    let _ = se.write_all(bytes);
    let _ = se.flush();
}

pub fn err_str(s: &str) {
    err(s.as_bytes());
}

/// An argument for [`cformat`].
#[derive(Clone, Copy)]
pub enum Arg<'a> {
    S(&'a [u8]),
    U(u64),
}

impl<'a> From<&'a [u8]> for Arg<'a> {
    fn from(s: &'a [u8]) -> Arg<'a> {
        Arg::S(s)
    }
}

impl<'a> From<&'a str> for Arg<'a> {
    fn from(s: &'a str) -> Arg<'a> {
        Arg::S(s.as_bytes())
    }
}

/// Minimal `printf` supporting the conversions used by the compiler's messages
/// (`%s %u %lu %d %llX %%`).
pub fn cformat(fmt: &str, args: &[Arg]) -> Vec<u8> {
    let f = fmt.as_bytes();
    let mut res = Vec::with_capacity(f.len() + 16);
    let mut args = args.iter();
    let mut i = 0;
    while i < f.len() {
        if f[i] != b'%' {
            res.push(f[i]);
            i += 1;
            continue;
        }
        i += 1;
        while i < f.len() && (f[i] == b'l' || f[i] == b'h' || f[i] == b'z') {
            i += 1;
        }
        let conv = f.get(i).copied().unwrap_or(b'%');
        i += 1;
        if conv == b'%' {
            res.push(b'%');
            continue;
        }
        let arg = args.next().copied().unwrap_or(Arg::S(b"(null)"));
        match (conv, arg) {
            (b's', Arg::S(s)) => res.extend_from_slice(s),
            (b'X', Arg::U(n)) => res.extend_from_slice(format!("{n:X}").as_bytes()),
            (_, Arg::U(n)) => res.extend_from_slice(n.to_string().as_bytes()),
            (_, Arg::S(s)) => res.extend_from_slice(s),
        }
    }
    res
}

fn platform_suffix(target: Target) -> &'static str {
    match target {
        Target::Windows => "Windows",
        Target::MacOS => "MacOS",
        Target::Linux => "Linux",
    }
}

const RANDOM_ERROR_MESSAGES: &[&str] = &[
    //sudo
    "you are not in the sudoers file. This incident will be reported.",
    "you are not allowed to run sudo. This incident will be reported.",
    //erno - cause why not?
    "ENOSPC: No space left on device",
    "ENOENT: No such file or directory",
    //glibc
    "double free or corruption (top)",
    "corrupted top size",
    "double free or corruption (!prev)",
    "free: invalid pointer",
    //gcc
    "implicit declaration of function `gets'",
    "invalid type argument of `unary *'",
    "passing arg 2 of `strcpy' makes pointer from integer without a cast",
    "syntax error before '}' token",
    "ld returned exit 1 status",
    "no match for ‘operator==’ in ‘__first.__gnu_cxx::__normal_iterator::operator* [with _Iterator = std::vector*, _Container = std::vector >, __gnu_cxx::__normal_iterator::reference = std::vector&]() == __val’",
    "error: cannot bind rvalue ‘(short unsigned int)((const char*)\"\")’ to ‘short unsigned int&’",
    "no match for 'operator<<' (operand types are 'std::ostream' {aka 'std::basic_ostream<char>'} and 'std::array<int, 1>')",
    "invalid conversion from `int' to `std::_Rb_tree_node<std::pair<const int, double> >*'",
    //ld
    "relocation truncated to fit: R_X86_64_PC32 against symbol `main'",
    "(.ARM.exidx.text._ZNSt8_Rb_treeIiSt4pairIKiSt10shared_ptrIN4SWGL7ContextEEESt10_Select1stIS6_ESt4lessIiESaIS6_EE13_Rb_tree_implISA_Lb1EED2Ev[_ZNSt8_Rb_treeIiSt4pairIKiSt10shared_ptrIN4SWGL7ContextEEESt10_Select1stIS6_ESt4lessIiESaIS6_EE13_Rb_tree_implISA_Lb1EED5Ev]+0x0): relocation truncated to fit: R_ARM_PREL31 against `.text._ZNSt8_Rb_treeIiSt4pairIKiSt10shared_ptrIN4SWGL7ContextEEESt10_Select1stIS6_ESt4lessIiESaIS6_EE13_Rb_tree_implISA_Lb1EED2Ev'",
    "error adding symbols: DSO missing from command line",
    "undefined reference to `main'",
    //Cobol
    "programmer is impolite",
    "programmer is excessively polite",
    //actual MemeASM errors
    "a decimal number cannot be a pointer",
    "a function name cannot be a pointer",
    "invalid parameter combination: 64 Bit arithmetic operation commands require the decimal number to be sign-extendable from 32 Bits",
    "function does not return",
    "an executable cannot be created if no main function exists",
];

/// Prints an ASCII-Art title and version information.
pub fn print_information_header(target: Target) {
    out_str(&format!("{BLU}  __  __                                                     _     _       \n"));
    out_str(" |  \\/  |                       /\\                          | |   | |      \n");
    out_str(" | \\  / | ___ _ __ ___   ___   /  \\   ___ ___  ___ _ __ ___ | |__ | |_   _ \n");
    out_str(" | |\\/| |/ _ \\ '_ ` _ \\ / _ \\ / /\\ \\ / __/ __|/ _ \\ '_ ` _ \\| '_ \\| | | | |\n");
    out_str(" | |  | |  __/ | | | | |  __// ____ \\\\__ \\__ \\  __/ | | | | | |_) | | |_| |\n");
    out_str(" |_|  |_|\\___|_| |_| |_|\\___/_/    \\_\\___/___/\\___|_| |_| |_|_.__/|_|\\__, |\n");
    out_str(&format!(
        "{RESET}  A Meme-based programming language.             {BLU}                     __/ |\n"
    ));
    out_str(&format!("                                                                     |___/ \n\n{RESET}"));
    out_str("For more information, a list of commands and code examples, please visit https://github.com/kammt/MemeAssembly.\n");
    out_str(&format!(
        "This is the MemeAssembly compiler {} ({}), created by Tobias Kamm.\n\n",
        VERSION_STRING,
        platform_suffix(target)
    ));
}

/// The "Wait, that's illegal!" ASCII art printed when compilation fails.
pub fn print_error_ascii() {
    out_str("\n");
    out_str("\n");
    out_str(&format!("{YEL}  __          __   _ _       _   _           _   _       _ _ _                  _ _  \n"));
    out_str(" \\ \\        / /  (_| |     | | | |         | | ( )     (_| | |                | | | \n");
    out_str("  \\ \\  /\\  / __ _ _| |_    | |_| |__   __ _| |_|/ ___   _| | | ___  __ _  __ _| | | \n");
    out_str("   \\ \\/  \\/ / _` | | __|   | __| '_ \\ / _` | __| / __| | | | |/ _ \\/ _` |/ _` | | | \n");
    out_str("    \\  /\\  | (_| | | |_ _  | |_| | | | (_| | |_  \\__ \\ | | | |  __| (_| | (_| | |_| \n");
    out_str("     \\/  \\/ \\__,_|_|\\__( )  \\__|_| |_|\\__,_|\\__| |___/ |_|_|_|\\___|\\__, |\\__,_|_(_) \n");
    out_str("                       |/                                           __/ |           \n");
    out_str(&format!("                                                                   |___/  \n{RESET}"));
}

/// The Snap ASCII art of "perfectly balanced as all things should be".
pub fn print_thanos_ascii(deleted_lines: usize) {
    out_str("\n");
    out_str("\n");
    out_str(&format!("{YEL}   _____                 \n"));
    out_str("  / ____|         \n");
    out_str(" | (___  _ __   __ _ _ __ \n");
    out_str("  \\___ \\| '_ \\ / _` | '_ \\ \n");
    out_str("  ____) | | | | (_| | |_) | \n");
    out_str(" |_____/|_| |_|\\__,_| .__/  \n");
    out_str("                    | |  \n");
    out_str(&format!("                    |_|    \n{RESET}"));
    out_str(&format!("{GRN}\nDid you do it?\n{RESET}"));
    out_str(&format!("{MAG}Yes\n{RESET}"));
    out_str(&format!("{GRN}What did it cost?\n{RESET}"));
    out_str(&format!("{MAG}{deleted_lines} lines of code\n\n{RESET}"));
}

/// Printed when a decimal parameter with value 420 or 69 is encountered.
pub fn print_nice_ascii() {
    out_str("\n");
    out_str("\n");
    out_str("\x1B[38;5;197m  _   _ _          \n");
    out_str("\x1B[38;5;197m | \\ | (_)         \n");
    out_str("\x1B[38;5;198m |  \\| |_  ___ ___ \n");
    out_str("\x1B[38;5;198m | . ` | |/ __/ _ \\\n");
    out_str("\x1B[38;5;199m | |\\  | | (_|  __/\n");
    out_str(&format!("\x1B[38;5;199m |_| \\_|_|\\___\\___|\n\n{RESET}"));
}

/// `printDebugMessage`.
pub fn print_debug(log_level: LogLevel, fmt: &str, args: &[Arg]) {
    if log_level == LogLevel::Debug {
        let mut m = cformat(fmt, args);
        m.push(b'\n');
        out(&m);
    }
}

/// `printError`. `fmt` is the C format string (its bytes feed the obfuscated-mode message
/// selection), `var_arg_num` the `varArgNum` the C call site passed.
pub fn print_error(
    state: &mut CompileState,
    input_file_name: &[u8],
    line_num: u64,
    fmt: &str,
    var_arg_num: u32,
    args: &[Arg],
) {
    state.compiler_errors += 1;
    let line_num = line_num as u32;

    let mut m = input_file_name.to_vec();
    m.extend_from_slice(format!(":{line_num}: {RED}error: {RESET}").as_bytes());

    if state.compile_mode != CompileMode::Obfuscated {
        m.extend_from_slice(&cformat(fmt, args));
    } else {
        let mut computed_index = line_num as u64;
        for &b in super::libc::cstr(input_file_name) {
            computed_index = computed_index.wrapping_add(u64::from(b));
        }
        for &b in fmt.as_bytes() {
            computed_index = computed_index.wrapping_add(u64::from(b));
        }
        computed_index = computed_index.wrapping_add(var_arg_num as u64);
        let idx = (computed_index % RANDOM_ERROR_MESSAGES.len() as u64) as usize;
        m.extend_from_slice(RANDOM_ERROR_MESSAGES[idx].as_bytes());
    }
    m.push(b'\n');
    out(&m);
}

/// `printNote`.
pub fn print_note(fmt: &str, indent: bool, args: &[Arg]) {
    let mut m = Vec::new();
    if indent {
        m.push(b'\t');
    }
    m.extend_from_slice(format!("{MAG}note: {RESET}").as_bytes());
    m.extend_from_slice(&cformat(fmt, args));
    m.push(b'\n');
    out(&m);
}

/// `printInternalCompilerError`.
pub fn print_internal_compiler_error(fmt: &str, report: bool, args: &[Arg]) {
    let mut m = format!("{RED}Internal compiler error: {RESET}").into_bytes();
    m.extend_from_slice(&cformat(fmt, args));
    m.push(b'\n');
    if report {
        m.extend_from_slice(b"Please report this error at https://github.com/kammt/MemeAssembly/issues/new");
    }
    err(&m);
}
