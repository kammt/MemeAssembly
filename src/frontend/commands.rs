//! Port of compiler/commands.h and the command table of compiler/compiler.c.

use super::encoder::{self as enc, Encode};
use crate::target::Target;

pub const NUMBER_OF_COMMANDS: usize = 46;
pub const OR_DRAW_25_OPCODE: u8 = (NUMBER_OF_COMMANDS - 2) as u8;
pub const INVALID_COMMAND_OPCODE: u8 = (NUMBER_OF_COMMANDS - 1) as u8;

// Parameter types
pub const PARAM_REG64: u8 = 1;
pub const PARAM_REG32: u8 = 2;
pub const PARAM_REG16: u8 = 4;
pub const PARAM_REG8: u8 = 8;
pub const PARAM_DECIMAL: u8 = 16;
pub const PARAM_CHAR: u8 = 32;
pub const PARAM_MONKE_LABEL: u8 = 64;
pub const PARAM_FUNC_NAME: u8 = 128;
pub const PARAM_REG: u8 = PARAM_REG64 | PARAM_REG32 | PARAM_REG16 | PARAM_REG8;

pub fn param_is_reg(param: u8) -> bool {
    param <= PARAM_REG8 && param > 0
}

// Command types
pub const COMMAND_TYPE_NONE: u8 = 0;
pub const COMMAND_TYPE_MOV: u8 = 1;
pub const COMMAND_TYPE_FUNC_RETURN: u8 = 2;
pub const COMMAND_TYPE_FUNC_DEF: u8 = 3;
pub const COMMAND_TYPE_FUNC_CALL: u8 = 4;

pub const COMMENT_START: &[u8] = b"What the hell happened here?";
pub const OR_DRAW_25_START: &[u8] = b"or";
pub const OR_DRAW_25_END: &[u8] = b"draw 25";
pub const POINTER_SUFFIX: &[u8] = b"do you know de wey";

/// The analysis function attached to a command (the C function pointer).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Analysis {
    Functions,
    Call,
    JumpMarkers,
    MonkeMarkers,
    WhoWouldWin,
    TheyreTheSamePicture,
    ConfusedStonks,
    PerfectlyBalanced,
}

pub struct Command {
    pub pattern: &'static str,
    pub command_type: u8,
    pub used_parameters: u8,
    pub allowed_param_types: [u8; 2],
    pub analysis: Option<Analysis>,
    /// Assembly text for `-S` (`{0}`/`{1}` = parameters, `{F}` = file number).
    pub translation_pattern: &'static str,
    /// Issues the same instructions as `translation_pattern` directly to the backend's emitter
    /// (object files and executables).
    pub encode: Encode,
}

const fn cmd(
    pattern: &'static str,
    command_type: u8,
    used_parameters: u8,
    allowed_param_types: [u8; 2],
    analysis: Option<Analysis>,
    translation_pattern: &'static str,
) -> Command {
    Command {
        pattern,
        command_type,
        used_parameters,
        allowed_param_types,
        analysis,
        translation_pattern,
        encode: enc::invalid,
    }
}

impl Command {
    const fn encode(self, encode: Encode) -> Command {
        Command { encode, ..self }
    }
}

const N: u8 = COMMAND_TYPE_NONE;

pub static COMMAND_LIST: [Command; NUMBER_OF_COMMANDS] = [
    // Functions
    cmd("I like to have fun, fun, fun, fun, fun, fun, fun, fun, fun, fun {p}", COMMAND_TYPE_FUNC_DEF, 1, [PARAM_FUNC_NAME, 0], Some(Analysis::Functions), "{0}:").encode(enc::func_def),
    cmd("right back at ya, buckaroo", COMMAND_TYPE_FUNC_RETURN, 0, [0, 0], None, "ret").encode(enc::ret),
    cmd("no, I don't think I will", COMMAND_TYPE_FUNC_RETURN, 0, [0, 0], None, "mov rax, 1\n\tret").encode(enc::ret_one),
    cmd("I see this as an absolute win", COMMAND_TYPE_FUNC_RETURN, 0, [0, 0], None, "xor rax, rax\n\tret").encode(enc::ret_zero),
    cmd("{p}: whomst has summoned the almighty one", COMMAND_TYPE_FUNC_CALL, 1, [PARAM_FUNC_NAME, 0], Some(Analysis::Call), "call {0}").encode(enc::call),
    // Stack operations
    cmd("stonks {p}", N, 1, [PARAM_REG64 | PARAM_DECIMAL | PARAM_CHAR, 0], None, "push {0}").encode(enc::push),
    cmd("not stonks {p}", N, 1, [PARAM_REG64, 0], None, "pop {0}").encode(enc::pop),
    // Logical Operations
    cmd("bitconneeeeeeect {p} {p}", N, 2, [PARAM_REG, PARAM_REG | PARAM_DECIMAL | PARAM_CHAR], None, "and {0}, {1}").encode(enc::and),
    cmd("{p} \\s", N, 1, [PARAM_REG, 0], None, "not {0}").encode(enc::not),
    // Register Manipulation
    cmd("sneak 100 {p}", N, 1, [PARAM_REG, 0], None, "xor {0}, {0}").encode(enc::zero),
    cmd("{p} is brilliant, but I like {p}", COMMAND_TYPE_MOV, 2, [PARAM_REG, PARAM_REG | PARAM_DECIMAL | PARAM_CHAR], None, "mov {0}, {1}").encode(enc::mov),
    // Arithmetic operations
    cmd("upvote {p}", N, 1, [PARAM_REG, 0], None, "add {0}, 1").encode(enc::upvote),
    cmd("downvote {p}", N, 1, [PARAM_REG, 0], None, "sub {0}, 1").encode(enc::downvote),
    cmd("parry {p} you filthy casual {p}", N, 2, [PARAM_REG | PARAM_DECIMAL | PARAM_CHAR, PARAM_REG], None, "sub {1}, {0}").encode(enc::parry),
    cmd("{p} units are ready, with {p} more well on the way", N, 2, [PARAM_REG, PARAM_REG | PARAM_DECIMAL | PARAM_CHAR], None, "add {0}, {1}").encode(enc::add),
    cmd("upgrades, people. Upgrades {p}", N, 1, [PARAM_REG, 0], None, "shl {0}, 1").encode(enc::shl),
    cmd("they had us in the first half, not gonna lie {p}", N, 1, [PARAM_REG, 0], None, "shr {0}, 1").encode(enc::shr),
    cmd("{p} is getting out of hand, now there are {p} of them", N, 2, [PARAM_REG64 | PARAM_REG32, PARAM_REG64 | PARAM_REG32 | PARAM_DECIMAL | PARAM_CHAR], None, "imul {0}, {1}").encode(enc::imul),
    cmd(
        "look at what {p} needs to mimic a fraction of {p}",
        N,
        2,
        [PARAM_REG64 | PARAM_DECIMAL | PARAM_CHAR, PARAM_REG64],
        None,
        "mov QWORD PTR [rip + .Ltmp64], {0}\n\t\
         push rdx\n\t\
         cqo\n\t\
         push rax\n\t\
         mov rax, {1}\n\t\
         idiv QWORD PTR [rip + .Ltmp64]\n\t\
         push rax\n\t\
         mov rax, [rsp + 8]\n\t\
         pop {1}\n\t\
         add rsp, 8\n\t\
         pop rdx\n\t",
    ).encode(enc::divide),
    cmd(
        "{p} UNLIMITED POWER {p}",
        N,
        2,
        [PARAM_REG64, PARAM_REG64 | PARAM_DECIMAL | PARAM_CHAR],
        None,
        "mov QWORD PTR [rip + .Ltmp64], {1}\n\t\
         cmp QWORD PTR [rip + .Ltmp64], 0\n\t\
         jne 2f\n\t\
         xor {0}, {0}\n\t\
         inc {0}\n\t\
         jmp 4f\n\t\
         2: push {0}\n\t\
         dec QWORD PTR [rip + .Ltmp64]\n\t\
         3: imul {0}, [rsp]\n\t\
         dec QWORD PTR [rip + .Ltmp64]\n\t\
         jnz 3b\n\t\
         add rsp, 8\n\t\
         4:\n\t",
    ).encode(enc::power),
    // Jumps and Jump Markers
    cmd("upgrade", N, 0, [0, 0], Some(Analysis::JumpMarkers), ".LUpgradeMarker_{F}:").encode(enc::upgrade_label),
    cmd("fuck go back", N, 0, [0, 0], None, "jmp .LUpgradeMarker_{F}").encode(enc::upgrade_jump),
    cmd("banana", N, 0, [0, 0], Some(Analysis::JumpMarkers), ".LBananaMarker_{F}:").encode(enc::banana_label),
    cmd("where banana", N, 0, [0, 0], None, "jmp .LBananaMarker_{F}").encode(enc::banana_jump),
    cmd("monke {p}", N, 1, [PARAM_MONKE_LABEL, 0], Some(Analysis::MonkeMarkers), ".L{0}:").encode(enc::monke_label),
    cmd("return to monke {p}", N, 1, [PARAM_MONKE_LABEL, 0], None, "jmp .L{0}").encode(enc::monke_jump),
    cmd("who would win? {p} or {p}", N, 2, [PARAM_REG, PARAM_REG | PARAM_DECIMAL | PARAM_CHAR], Some(Analysis::WhoWouldWin), "cmp {0}, {1}\n\tjg .L{0}Wins_{F}\n\tjl .L{1}Wins_{F}").encode(enc::who_would_win),
    cmd("{p} wins", N, 1, [PARAM_REG | PARAM_DECIMAL | PARAM_CHAR, 0], None, ".L{0}Wins_{F}:").encode(enc::wins_label),
    cmd("corporate needs you to find the difference between {p} and {p}", N, 2, [PARAM_REG, PARAM_REG | PARAM_DECIMAL | PARAM_CHAR], Some(Analysis::TheyreTheSamePicture), "cmp {0}, {1}\n\tje .LSamePicture_{F}").encode(enc::find_difference),
    cmd("they're the same picture", N, 0, [0, 0], None, ".LSamePicture_{F}:").encode(enc::same_picture_label),
    // IO-Operations
    cmd(
        "what can I say except {p}",
        N,
        1,
        [PARAM_REG8 | PARAM_CHAR, 0],
        None,
        "mov BYTE PTR [rip + .LCharacter], {0}\n\t\
         test rsp, 0xF\n\t\
         jz 1f\n\t\
         sub rsp, 8\n\t\
         call writechar\n\t\
         add rsp, 8\n\t\
         jmp 2f\n\t\
         1: call writechar\n\t\
         2:\n\t",
    ).encode(enc::write_char),
    cmd(
        "let me in. LET ME IIIIIIIIN {p}",
        N,
        1,
        [PARAM_REG8, 0],
        None,
        "test rsp, 0xF\n\t\
         jz 1f\n\t\
         sub rsp, 8\n\t\
         call readchar\n\t\
         add rsp, 8\n\t\
         jmp 2f\n\t\
         1: call readchar\n\t\
         2:\n\t\
         mov {0}, BYTE PTR [rip + .LCharacter]\n\t",
    ).encode(enc::read_char),
    // Random commands
    cmd("guess I'll die", N, 0, [0, 0], None, "mov rax, [69]").encode(enc::die),
    cmd("confused stonks", N, 0, [0, 0], Some(Analysis::ConfusedStonks), "jmp .LConfusedStonks_{F}").encode(enc::confused_stonks),
    cmd("perfectly balanced as all things should be", N, 0, [0, 0], Some(Analysis::PerfectlyBalanced), "").encode(enc::perfectly_balanced),
    cmd("wait, that's illegal", N, 0, [0, 0], None, "xor rbx, rbx\n\txor rbp, rbp\n\txor r12, r12\n\txor r13 r13").encode(enc::illegal),
    cmd("oh no! anyway", N, 0, [0, 0], None, "nop").encode(enc::oh_no),
    cmd("it's over 9000 {p}", N, 1, [PARAM_REG, 0], None, "cmp {0}, 9000\n\tjg 1f\n\thlt\n\t1:").encode(enc::over_9000),
    cmd("refuses to elaborate and leaves", N, 0, [0, 0], None, "mov rbp, rsp\n\tpop rsp").encode(enc::refuses_to_elaborate),
    cmd("you shall not pass!", N, 0, [0, 0], None, "1: xor rax, rax\n\tjmp 1b").encode(enc::shall_not_pass),
    cmd("Houston, we have a problem", N, 0, [0, 0], None, "xor rsp, rsp").encode(enc::houston),
    cmd("it's dangerous to go alone, take {p}", N, 1, [PARAM_REG64 | PARAM_REG32 | PARAM_REG16, 0], None, "rdrand {0}").encode(enc::rdrand),
    cmd("we need air support", N, 0, [0, 0], None, "syscall").encode(enc::air_support),
    // Debug commands
    cmd("it's a trap", N, 0, [0, 0], None, "int3").encode(enc::trap),
    //Insert commands above this one
    cmd("or draw 25", N, 0, [0, 0], None, "add eax, 25").encode(enc::draw_25),
    cmd("", N, 0, [0, 0], None, "0").encode(enc::invalid),
];

/// Function names used for bully-mode replacements.
pub const FUNCTION_NAMES: [&str; 16] = [
    "mprotect", "kill", "signal", "raise", "dump", "atoi", "generateExcellence", "isExcellent",
    "memeify", "uwufy", "test", "helloWorld", "snake_case_sucks", "gets", "uwu", "skillIssue",
];

#[derive(Debug, Clone)]
pub struct ParsedCommand {
    pub opcode: u8,
    /// `None` stands for a parameter slot the C code never initialised for this command.
    pub parameters: [Option<Vec<u8>>; 2],
    pub param_types: [u8; 2],
    /// 0 = no pointer, 1 = first parameter, 2 = second parameter
    pub is_pointer: u8,
    pub line_num: usize,
    /// Set to false if the command must not be translated
    pub translate: bool,
}

impl ParsedCommand {
    pub fn new(line_num: usize) -> ParsedCommand {
        ParsedCommand {
            opcode: 0,
            parameters: [None, None],
            param_types: [0, 0],
            is_pointer: 0,
            line_num,
            translate: true,
        }
    }

    /// Parameter `i` as a C string (an uninitialised slot reads as an empty string).
    pub fn param(&self, i: usize) -> &[u8] {
        self.parameters[i].as_deref().unwrap_or(b"")
    }

    /// `paramTypes[i]` including the out-of-bounds read of `paramTypes[2]`, which lands on
    /// `isPointer` in the C struct layout.
    pub fn param_type_c(&self, i: usize) -> u8 {
        match i {
            0 | 1 => self.param_types[i],
            _ => self.is_pointer,
        }
    }
}

/// Where the commands of a function live. Regular functions alias a slice of the file's
/// command array; bully mode's injected functions own a copy (exactly like the C code, which
/// matters for which array "perfectly balanced" modifies).
#[derive(Debug, Clone)]
pub enum FunctionCommands {
    InFile(usize),
    Owned(Vec<ParsedCommand>),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub number_of_commands: usize,
    pub commands: FunctionCommands,
}

#[derive(Debug, Clone)]
pub struct File {
    pub file_name: Vec<u8>,
    /// lines of code
    pub loc: usize,
    pub functions: Vec<Function>,
    pub parsed_commands: Vec<ParsedCommand>,
    /// A variable necessary for the "confused stonks" command
    pub random_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// `optimisationLevel` values. Note that in the C enum `o_s` follows `o_3 = -3` and therefore
/// has the value -2, i.e. it is the same as `o_2`.
pub const OPT_NONE: i64 = 0;
pub const OPT_1: i64 = -1;
pub const OPT_2: i64 = -2;
pub const OPT_3: i64 = -3;
pub const OPT_S: i64 = OPT_3 + 1;
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

    pub fn function_commands(&self, file: usize, func: usize) -> &[ParsedCommand] {
        let f = &self.files[file];
        let function = &f.functions[func];
        match &function.commands {
            FunctionCommands::InFile(start) => {
                &f.parsed_commands[*start..*start + function.number_of_commands]
            }
            FunctionCommands::Owned(v) => v,
        }
    }

    pub fn cmd(&self, l: CmdLoc) -> &ParsedCommand {
        &self.function_commands(l.file, l.func)[l.k]
    }

    pub fn cmd_mut(&mut self, l: CmdLoc) -> &mut ParsedCommand {
        let f = &mut self.files[l.file];
        match &mut f.functions[l.func].commands {
            FunctionCommands::InFile(start) => &mut f.parsed_commands[*start + l.k],
            FunctionCommands::Owned(v) => &mut v[l.k],
        }
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
