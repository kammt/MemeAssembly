//! Port of compiler/commands.h and the command table of compiler/compiler.c.

use super::translator::{self as tr, CodeGen};
use crate::backend::emitter::Reg;
use std::borrow::Cow;

use crate::target::Target;

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

/// A command: its index in [`COMMAND_LIST`] (the C opcode).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    FuncDef, Return, ReturnOne, ReturnZero, Call,
    Push, Pop,
    And, Not,
    Zero, Mov,
    Upvote, Downvote, Parry, Add, Shl, Shr, Imul, Divide, Power,
    Upgrade, FuckGoBack, Banana, WhereBanana, Monke, ReturnToMonke, WhoWouldWin, Wins,
    FindDifference, SamePicture,
    WriteChar, ReadChar,
    Die, ConfusedStonks, PerfectlyBalanced, Illegal, OhNo, Over9000, RefusesToElaborate,
    ShallNotPass, Houston, Rdrand, AirSupport,
    Trap,
    /// "or draw 25" at the end of a line replaces the command
    OrDraw25,
    /// A line that is no command
    Invalid,
}

impl Opcode {
    pub fn command(self) -> &'static Command {
        &COMMAND_LIST[self as usize]
    }
}

pub struct Command {
    pub opcode: Opcode,
    pub pattern: &'static str,
    pub command_type: u8,
    pub used_parameters: u8,
    pub allowed_param_types: [u8; 2],
    /// Generates the command's code (for `-S` text and object code alike).
    pub code: CodeGen,
}

const fn cmd(
    opcode: Opcode,
    pattern: &'static str,
    command_type: u8,
    used_parameters: u8,
    allowed_param_types: [u8; 2],
    code: CodeGen,
) -> Command {
    Command { opcode, pattern, command_type, used_parameters, allowed_param_types, code }
}

const N: u8 = COMMAND_TYPE_NONE;

pub static COMMAND_LIST: [Command; 46] = [
    // Functions
    cmd(Opcode::FuncDef, "I like to have fun, fun, fun, fun, fun, fun, fun, fun, fun, fun {p}", COMMAND_TYPE_FUNC_DEF, 1, [PARAM_FUNC_NAME, 0], tr::func_def),
    cmd(Opcode::Return, "right back at ya, buckaroo", COMMAND_TYPE_FUNC_RETURN, 0, [0, 0], tr::ret),
    cmd(Opcode::ReturnOne, "no, I don't think I will", COMMAND_TYPE_FUNC_RETURN, 0, [0, 0], tr::ret_one),
    cmd(Opcode::ReturnZero, "I see this as an absolute win", COMMAND_TYPE_FUNC_RETURN, 0, [0, 0], tr::ret_zero),
    cmd(Opcode::Call, "{p}: whomst has summoned the almighty one", COMMAND_TYPE_FUNC_CALL, 1, [PARAM_FUNC_NAME, 0], tr::call),
    // Stack operations
    cmd(Opcode::Push, "stonks {p}", N, 1, [PARAM_REG64 | PARAM_DECIMAL | PARAM_CHAR, 0], tr::push),
    cmd(Opcode::Pop, "not stonks {p}", N, 1, [PARAM_REG64, 0], tr::pop),
    // Logical Operations
    cmd(Opcode::And, "bitconneeeeeeect {p} {p}", N, 2, [PARAM_REG, PARAM_REG | PARAM_DECIMAL | PARAM_CHAR], tr::and),
    cmd(Opcode::Not, "{p} \\s", N, 1, [PARAM_REG, 0], tr::not),
    // Register Manipulation
    cmd(Opcode::Zero, "sneak 100 {p}", N, 1, [PARAM_REG, 0], tr::zero),
    cmd(Opcode::Mov, "{p} is brilliant, but I like {p}", COMMAND_TYPE_MOV, 2, [PARAM_REG, PARAM_REG | PARAM_DECIMAL | PARAM_CHAR], tr::mov),
    // Arithmetic operations
    cmd(Opcode::Upvote, "upvote {p}", N, 1, [PARAM_REG, 0], tr::upvote),
    cmd(Opcode::Downvote, "downvote {p}", N, 1, [PARAM_REG, 0], tr::downvote),
    cmd(Opcode::Parry, "parry {p} you filthy casual {p}", N, 2, [PARAM_REG | PARAM_DECIMAL | PARAM_CHAR, PARAM_REG], tr::parry),
    cmd(Opcode::Add, "{p} units are ready, with {p} more well on the way", N, 2, [PARAM_REG, PARAM_REG | PARAM_DECIMAL | PARAM_CHAR], tr::add),
    cmd(Opcode::Shl, "upgrades, people. Upgrades {p}", N, 1, [PARAM_REG, 0], tr::shl),
    cmd(Opcode::Shr, "they had us in the first half, not gonna lie {p}", N, 1, [PARAM_REG, 0], tr::shr),
    cmd(Opcode::Imul, "{p} is getting out of hand, now there are {p} of them", N, 2, [PARAM_REG64 | PARAM_REG32, PARAM_REG64 | PARAM_REG32 | PARAM_DECIMAL | PARAM_CHAR], tr::imul),
    cmd(Opcode::Divide, "look at what {p} needs to mimic a fraction of {p}", N, 2, [PARAM_REG64 | PARAM_DECIMAL | PARAM_CHAR, PARAM_REG64], tr::divide),
    cmd(Opcode::Power, "{p} UNLIMITED POWER {p}", N, 2, [PARAM_REG64, PARAM_REG64 | PARAM_DECIMAL | PARAM_CHAR], tr::power),
    // Jumps and Jump Markers
    cmd(Opcode::Upgrade, "upgrade", N, 0, [0, 0], tr::upgrade_label),
    cmd(Opcode::FuckGoBack, "fuck go back", N, 0, [0, 0], tr::upgrade_jump),
    cmd(Opcode::Banana, "banana", N, 0, [0, 0], tr::banana_label),
    cmd(Opcode::WhereBanana, "where banana", N, 0, [0, 0], tr::banana_jump),
    cmd(Opcode::Monke, "monke {p}", N, 1, [PARAM_MONKE_LABEL, 0], tr::monke_label),
    cmd(Opcode::ReturnToMonke, "return to monke {p}", N, 1, [PARAM_MONKE_LABEL, 0], tr::monke_jump),
    cmd(Opcode::WhoWouldWin, "who would win? {p} or {p}", N, 2, [PARAM_REG, PARAM_REG | PARAM_DECIMAL | PARAM_CHAR], tr::who_would_win),
    cmd(Opcode::Wins, "{p} wins", N, 1, [PARAM_REG | PARAM_DECIMAL | PARAM_CHAR, 0], tr::wins_label),
    cmd(Opcode::FindDifference, "corporate needs you to find the difference between {p} and {p}", N, 2, [PARAM_REG, PARAM_REG | PARAM_DECIMAL | PARAM_CHAR], tr::find_difference),
    cmd(Opcode::SamePicture, "they're the same picture", N, 0, [0, 0], tr::same_picture_label),
    // IO-Operations
    cmd(Opcode::WriteChar, "what can I say except {p}", N, 1, [PARAM_REG8 | PARAM_CHAR, 0], tr::write_char),
    cmd(Opcode::ReadChar, "let me in. LET ME IIIIIIIIN {p}", N, 1, [PARAM_REG8, 0], tr::read_char),
    // Random commands
    cmd(Opcode::Die, "guess I'll die", N, 0, [0, 0], tr::die),
    cmd(Opcode::ConfusedStonks, "confused stonks", N, 0, [0, 0], tr::confused_stonks),
    cmd(Opcode::PerfectlyBalanced, "perfectly balanced as all things should be", N, 0, [0, 0], tr::perfectly_balanced),
    cmd(Opcode::Illegal, "wait, that's illegal", N, 0, [0, 0], tr::illegal),
    cmd(Opcode::OhNo, "oh no! anyway", N, 0, [0, 0], tr::oh_no),
    cmd(Opcode::Over9000, "it's over 9000 {p}", N, 1, [PARAM_REG, 0], tr::over_9000),
    cmd(Opcode::RefusesToElaborate, "refuses to elaborate and leaves", N, 0, [0, 0], tr::refuses_to_elaborate),
    cmd(Opcode::ShallNotPass, "you shall not pass!", N, 0, [0, 0], tr::shall_not_pass),
    cmd(Opcode::Houston, "Houston, we have a problem", N, 0, [0, 0], tr::houston),
    cmd(Opcode::Rdrand, "it's dangerous to go alone, take {p}", N, 1, [PARAM_REG64 | PARAM_REG32 | PARAM_REG16, 0], tr::rdrand),
    cmd(Opcode::AirSupport, "we need air support", N, 0, [0, 0], tr::air_support),
    // Debug commands
    cmd(Opcode::Trap, "it's a trap", N, 0, [0, 0], tr::trap),
    //Insert commands above this one
    cmd(Opcode::OrDraw25, "or draw 25", N, 0, [0, 0], tr::draw_25),
    cmd(Opcode::Invalid, "", N, 0, [0, 0], tr::invalid),
];

// Every command is at the index of its opcode.
const _: () = {
    let mut i = 0;
    while i < COMMAND_LIST.len() {
        assert!(COMMAND_LIST[i].opcode as usize == i);
        i += 1;
    }
};

/// Function names used for bully-mode replacements.
pub const FUNCTION_NAMES: [&str; 16] = [
    "mprotect", "kill", "signal", "raise", "dump", "atoi", "generateExcellence", "isExcellent",
    "memeify", "uwufy", "test", "helloWorld", "snake_case_sucks", "gets", "uwu", "skillIssue",
];

/// A command parameter. The parser stores the text (`Raw`), the analyser classifies it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Param {
    /// Unclassified text: what the parser read, or an invalid parameter. `mangle`: the
    /// parameter of a function definition or call, which gets a `_` prefix on macOS.
    Raw { text: Vec<u8>, mangle: bool },
    Reg(Reg),
    /// A decimal number and its source text (shown in diagnostics; bully mode cuts digits off).
    Decimal { value: i64, text: Vec<u8> },
    /// A character with its value and C text (`'a'`, `'\n'`, `' '` or an ASCII code like `65`).
    /// `verbatim`: gas reads `text` as `value` (not so for `'\v'`, `'\'` and ASCII codes with
    /// leading zeros, which gas reads as octal), so the `-S` text can show it.
    Char { value: u8, text: Vec<u8>, verbatim: bool },
    MonkeLabel(Vec<u8>),
    FuncName { name: Vec<u8>, mangle: bool },
}

impl Param {
    /// The parameter as the C code had it in its string: shown in diagnostics and compared by
    /// the analyser (function names with their `_` prefix on macOS).
    pub fn text(&self, target: Target) -> Cow<'_, [u8]> {
        match self {
            Param::Raw { text, mangle } | Param::FuncName { name: text, mangle } => {
                if *mangle && target == Target::MacOS {
                    Cow::Owned([b"_", &text[..]].concat())
                } else {
                    Cow::Borrowed(text)
                }
            }
            Param::Reg(r) => Cow::Owned(r.name().into_bytes()),
            Param::Decimal { text, .. } | Param::Char { text, .. } | Param::MonkeLabel(text) => {
                Cow::Borrowed(text)
            }
        }
    }

    /// The `PARAM_*` type (0 = not classified / invalid).
    pub fn param_type(&self) -> u8 {
        match self {
            Param::Raw { .. } | Param::Reg(Reg::Xmm(_)) => 0,
            Param::Reg(Reg::R64(_)) => PARAM_REG64,
            Param::Reg(Reg::R32(_)) => PARAM_REG32,
            Param::Reg(Reg::R16(_)) => PARAM_REG16,
            Param::Reg(Reg::R8(_) | Reg::R8H(_)) => PARAM_REG8,
            Param::Decimal { .. } => PARAM_DECIMAL,
            Param::Char { .. } => PARAM_CHAR,
            Param::MonkeLabel(_) => PARAM_MONKE_LABEL,
            Param::FuncName { .. } => PARAM_FUNC_NAME,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParsedCommand {
    pub opcode: Opcode,
    /// `None` stands for a parameter slot the C code never initialised for this command.
    pub parameters: [Option<Param>; 2],
    /// 0 = no pointer, 1 = first parameter, 2 = second parameter
    pub is_pointer: u8,
    pub line_num: usize,
    /// Set to false if the command must not be translated
    pub translate: bool,
}

impl ParsedCommand {
    pub fn new(line_num: usize) -> ParsedCommand {
        ParsedCommand { opcode: Opcode::Invalid, parameters: [None, None], is_pointer: 0, line_num, translate: true }
    }

    /// The text of parameter `i` (see [`Param::text`]; an uninitialised slot reads as an empty
    /// string).
    pub fn param_text(&self, i: usize, target: Target) -> Cow<'_, [u8]> {
        match &self.parameters[i] {
            Some(p) => p.text(target),
            None => Cow::Borrowed(b""),
        }
    }

    /// The `PARAM_*` type of parameter `i`.
    pub fn param_type(&self, i: usize) -> u8 {
        self.parameters[i].as_ref().map_or(0, Param::param_type)
    }

    /// In bully mode, a pointer operand gets a (random) operand size if nothing else determines
    /// it: for commands whose other operand gives the size, only if that operand is not a
    /// register; always for the one-operand commands that need a size.
    ///
    /// The C code checked `paramTypes[index + 1 % 2]` (= `paramTypes[index + 1]`, reading past
    /// the array for the second parameter); this is the intended `(index + 1) % 2`.
    pub fn bully_pointer_needs_size(&self, index: usize) -> bool {
        match pointer_use(self.opcode, index) {
            PointerUse::Sized => !param_is_reg(self.param_type((index + 1) % 2)),
            PointerUse::NeedsSize => true,
            PointerUse::Implicit | PointerUse::Unsupported => false,
        }
    }
}

/// How the generated code uses parameter `index` when it is a pointer (`[reg]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerUse {
    /// The other operand (a register) gives the operand size, e.g. `mov [rax], bl`.
    Sized,
    /// The instruction implies the size (`push [rax]`, `pop [rax]`).
    Implicit,
    /// A one-operand instruction (`not [rax]`) or one with an immediate (`add [rax], 1`): the
    /// size is ambiguous.
    NeedsSize,
    /// Not encodable: memory-to-memory, a memory destination for `imul`/`rdrand`, a pointer in
    /// a label name, ...
    Unsupported,
}

pub fn pointer_use(opcode: Opcode, index: usize) -> PointerUse {
    use Opcode::*;
    match (opcode, index) {
        (And | Mov | Parry | Add | FindDifference, _) => PointerUse::Sized,
        // imul: only the source can be memory
        (Imul, 1) => PointerUse::Sized,
        // the dividend of the division is pushed and popped
        (Push | Pop, 0) | (Divide, 1) => PointerUse::Implicit,
        (Not | Upvote | Downvote | Shl | Shr | Over9000, 0) => PointerUse::NeedsSize,
        _ => PointerUse::Unsupported,
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
/// distinct, as the help page describes them (see docs/rust-rewrite-changes.md).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameter_text_and_type() {
        let f = Param::FuncName { name: b"main".to_vec(), mangle: true };
        assert_eq!(&*f.text(Target::MacOS), b"_main");
        assert_eq!(&*f.text(Target::Linux), b"main");
        let f = Param::FuncName { name: b"kill".to_vec(), mangle: false };
        assert_eq!(&*f.text(Target::MacOS), b"kill");
        assert_eq!(&*Param::Reg(Reg::R8H(0)).text(Target::Linux), b"ah");
        assert_eq!(Param::Reg(Reg::R8H(0)).param_type(), PARAM_REG8);
        let c = Param::Char { value: 10, text: b"'\\n'".to_vec(), verbatim: true };
        assert_eq!((&*c.text(Target::Linux), c.param_type()), (&b"'\\n'"[..], PARAM_CHAR));
        assert_eq!(Param::Raw { text: b"x".to_vec(), mangle: true }.param_type(), 0);
    }
}
