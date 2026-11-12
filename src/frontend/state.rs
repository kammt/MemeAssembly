//! The compilation state: options, the parsed files and their functions.

use super::commands::ParsedCommand;
use crate::target::Target;

#[derive(Debug, Clone)]
pub struct Function {
    /// The definition (`commands[0]`, which holds the name) and the rest of the function.
    pub commands: Vec<ParsedCommand>,
    /// A function that bully mode made up for orphaned commands: its definition and return
    /// statement are not lines of the file.
    pub injected: bool,
}

impl Function {
    /// The commands that are lines of the file.
    fn lines_mut(&mut self) -> &mut [ParsedCommand] {
        let n = self.commands.len();
        if self.injected {
            &mut self.commands[1..n - 1]
        } else {
            &mut self.commands
        }
    }
}

#[derive(Debug, Clone)]
pub struct File {
    pub file_name: Vec<u8>,
    /// lines of code
    pub loc: usize,
    pub functions: Vec<Function>,
    /// Commands outside of any function (an error; bully mode puts them into injected
    /// functions instead).
    pub orphans: Vec<ParsedCommand>,
    /// A variable necessary for the "confused stonks" command
    pub random_index: usize,
}

impl File {
    /// The commands of the file's `loc` lines of code, in source order.
    pub fn lines_mut(&mut self) -> Vec<&mut ParsedCommand> {
        let mut lines: Vec<_> =
            self.functions.iter_mut().flat_map(Function::lines_mut).chain(&mut self.orphans).collect();
        lines.sort_by_key(|c| c.line_num);
        lines
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum CompileMode {
    Noob,
    Bully,
    Obfuscated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Executable,
    AssemblyFile,
    ObjectFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Normal,
    Debug,
}

/// `optimisationLevel` values. In the C enum `o_s` followed `o_3 = -3` and therefore had the
/// value -2 (the same as `o_2`), so `-O-2` and `-O-s` both did both things. Here they are
/// distinct, as the help page describes them.
pub const OPT_NONE: i64 = 0;
pub const OPT_1: i64 = -1;
pub const OPT_2: i64 = -2;
pub const OPT_3: i64 = -3;
pub const OPT_S: i64 = -4;
pub const OPT_69420: i64 = 69420;

/// Location of a command: file index, function index, command index within the function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CmdLoc {
    pub file: usize,
    pub func: usize,
    pub k: usize,
}

pub struct CompileState {
    pub compile_mode: CompileMode,
    pub output_mode: OutputMode,
    pub files: Vec<File>,
    /// `-g`: emit debug information (DWARF; the C compiler wrote STABS).
    pub use_debug_info: bool,
    pub martyrdom: bool,
    pub optimisation_level: i64,
    pub compiler_errors: u32,
    pub log_level: LogLevel,
    pub target: Target,
    /// The C global `computedIndex` used for bully-mode pseudo-randomness.
    pub computed_index: u64,
}

impl CompileState {
    pub fn new(target: Target) -> CompileState {
        CompileState {
            compile_mode: CompileMode::Noob,
            output_mode: OutputMode::Executable,
            files: Vec::new(),
            use_debug_info: false,
            martyrdom: true,
            optimisation_level: OPT_NONE,
            compiler_errors: 0,
            log_level: LogLevel::Normal,
            target,
            computed_index: 69,
        }
    }

    pub fn bully(&self) -> bool {
        self.compile_mode == CompileMode::Bully
    }

    pub fn cmd(&self, l: CmdLoc) -> &ParsedCommand {
        &self.files[l.file].functions[l.func].commands[l.k]
    }

    pub fn cmd_mut(&mut self, l: CmdLoc) -> &mut ParsedCommand {
        &mut self.files[l.file].functions[l.func].commands[l.k]
    }

    /// Name of the `main` function on the current target.
    pub fn main_function_name(&self) -> &'static [u8] {
        if self.target == Target::MacOS {
            b"_main"
        } else {
            b"main"
        }
    }
}
