//! Console output: writing to stdout/stderr, the ASCII-art banners, notes, debug messages and
//! compile errors (including the made-up messages of obfuscated mode). Compile errors, notes
//! and debug messages go to stdout; I/O failures and internal errors go to stderr.

use std::io::{ErrorKind, Write};

use super::state::{CompileMode, CompileState, LogLevel};
use crate::target::Target;

pub const RED: &str = "\x1B[31m";
pub const GRN: &str = "\x1B[32m";
pub const YEL: &str = "\x1B[33m";
pub const BLU: &str = "\x1B[34m";
pub const MAG: &str = "\x1B[35m";
pub const RESET: &str = "\x1B[0m";

/// The compiler version, shown in the help header and by `-v` (without the `v`), and written
/// into the `-S` header and the debug information.
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
        err_str(&format!("Failed to write to stdout: {}\n", io_error_message(&e)));
        std::process::exit(1);
    }
}

/// The text of an I/O error without Rust's ` (os error N)` suffix.
pub fn io_error_message(e: &std::io::Error) -> String {
    strip_os_error(&e.to_string())
}

/// Removes Rust's ` (os error N)` suffix, leaving a perror-style `context: message`.
pub(super) fn strip_os_error(s: &str) -> String {
    match s.rfind(" (os error ") {
        Some(p) => s[..p].to_string(),
        None => s.to_string(),
    }
}

/// [`out`] for text.
pub fn out_str(s: &str) {
    out(s.as_bytes());
}

/// Writes raw bytes to stderr (write errors are ignored).
pub fn err(bytes: &[u8]) {
    let mut se = std::io::stderr().lock();
    let _ = se.write_all(bytes);
    let _ = se.flush();
}

/// [`err`] for text.
pub fn err_str(s: &str) {
    err(s.as_bytes());
}

fn platform_suffix(target: Target) -> &'static str {
    match target {
        Target::Windows => "Windows",
        Target::MacOS => "MacOS",
        Target::Linux => "Linux",
    }
}

/// The fake error messages that obfuscated mode prints instead of the real ones.
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

/// Prints a debug message if `-d` was given. Debug messages start with "debug: ".
pub fn print_debug(log_level: LogLevel, message: impl AsRef<[u8]>) {
    if log_level == LogLevel::Debug {
        out(&[b"debug: ", message.as_ref(), b"\n"].concat());
    }
}

/// Selects the fake message that obfuscated mode shows instead of an error: the sum of the bytes
/// of the error's printf-style format string (with its `%s`/`%u`/`%lu`, not the substituted
/// text) plus `argument_count`. It depends on the kind of error, not on its parameters;
/// [`print_error_with_key`] adds the file name and line number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorKey(u64);

impl ErrorKey {
    pub const fn new(c_format: &str, argument_count: u64) -> ErrorKey {
        let bytes = c_format.as_bytes();
        let mut sum = argument_count;
        let mut i = 0;
        while i < bytes.len() {
            sum += bytes[i] as u64;
            i += 1;
        }
        ErrorKey(sum)
    }
}

/// Reports a compile error without parameters (its text is also its format string for the
/// [`ErrorKey`]).
pub fn print_error(state: &mut CompileState, input_file_name: &[u8], line_num: usize, message: &str) {
    print_error_with_key(state, input_file_name, line_num, ErrorKey::new(message, 0), message.as_bytes());
}

/// Reports a compile error with parameters: `c_format` is a printf-style format string whose
/// `%s`, `%u` and `%lu` are replaced by `args` in order. The obfuscated-mode key counts `args`.
pub fn print_error_fmt(state: &mut CompileState, input_file_name: &[u8], line_num: usize, c_format: &str, args: &[&[u8]]) {
    let key = ErrorKey::new(c_format, args.len() as u64);
    print_error_with_key(state, input_file_name, line_num, key, &format_c(c_format, args));
}

/// Replaces the `%s`, `%u` and `%lu` of `c_format` by `args`, in order.
pub fn format_c(c_format: &str, args: &[&[u8]]) -> Vec<u8> {
    let mut message = Vec::new();
    let mut args = args.iter();
    let mut rest = c_format;
    while let Some(i) = rest.find('%') {
        message.extend_from_slice(&rest.as_bytes()[..i]);
        rest = &rest[i..];
        let spec = ["%s", "%u", "%lu"].into_iter().find(|s| rest.starts_with(s)).expect("supported conversion");
        message.extend_from_slice(args.next().expect("an argument for every conversion"));
        rest = &rest[spec.len()..];
    }
    message.extend_from_slice(rest.as_bytes());
    assert!(args.next().is_none(), "an argument without a conversion in {c_format:?}");
    message
}

/// Reports a compile error as `file:line: error: message` on stdout and counts it in
/// [`CompileState::compiler_errors`]. In obfuscated mode a fake message, chosen from the file
/// name, the line number and `key`, replaces `message`.
pub fn print_error_with_key(
    state: &mut CompileState,
    input_file_name: &[u8],
    line_num: usize,
    key: ErrorKey,
    message: &[u8],
) {
    state.compiler_errors += 1;
    let line_num = line_num as u32;

    let mut m = input_file_name.to_vec();
    m.extend_from_slice(format!(":{line_num}: {RED}error: {RESET}").as_bytes());

    if state.compile_mode != CompileMode::Obfuscated {
        m.extend_from_slice(message);
    } else {
        let mut computed_index = u64::from(line_num);
        for &b in input_file_name {
            computed_index = computed_index.wrapping_add(u64::from(b));
        }
        computed_index = computed_index.wrapping_add(key.0);
        let idx = (computed_index % RANDOM_ERROR_MESSAGES.len() as u64) as usize;
        m.extend_from_slice(RANDOM_ERROR_MESSAGES[idx].as_bytes());
    }
    m.push(b'\n');
    out(&m);
}

/// Prints `note: message` to stdout, indented by a tab if `indent`.
pub fn print_note(message: &str, indent: bool) {
    let tab = if indent { "\t" } else { "" };
    out_str(&format!("{tab}{MAG}note: {RESET}{message}\n"));
}

/// Prints an internal compiler error to stderr, with a request to report it if `report`.
pub fn print_internal_compiler_error(message: &str, report: bool) {
    let mut m = format!("{RED}Internal compiler error: {RESET}{message}\n");
    if report {
        m.push_str("Please report this error at https://github.com/kammt/MemeAssembly/issues/new");
    }
    err_str(&m);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_format() {
        assert_eq!(format_c("no args", &[]), b"no args");
        let message = format_c("'%s' (%u bits) in %s:%lu", &[b"x", b"9", b"f", b"3"]);
        assert_eq!(message, b"'x' (9 bits) in f:3");
    }
}
