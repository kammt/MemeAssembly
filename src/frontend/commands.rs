//! The commands of MemeAssembly: their patterns, parameter types and code generators. The
//! parser matches lines against the patterns of [`COMMAND_LIST`], the analyser checks the
//! parameters against `allowed_param_types`, and the translator runs each command's `code`.

use super::translator::{self as tr, CodeGen};
use crate::backend::emitter::Reg;
use std::borrow::Cow;

use crate::target::Target;

// Parameter types (bit flags)
pub const PARAM_REG64: u16 = 1;
pub const PARAM_REG32: u16 = 2;
pub const PARAM_REG16: u16 = 4;
pub const PARAM_REG8: u16 = 8;
pub const PARAM_DECIMAL: u16 = 16;
pub const PARAM_CHAR: u16 = 32;
pub const PARAM_MONKE_LABEL: u16 = 64;
pub const PARAM_FUNC_NAME: u16 = 128;
/// The edition of a "who would win" comparison (`(2026 edition)`, `2026: rax wins`).
pub const PARAM_COMPARISON_LABEL: u16 = 256;
pub const PARAM_REG: u16 = PARAM_REG64 | PARAM_REG32 | PARAM_REG16 | PARAM_REG8;
/// An immediate: a decimal number or a character.
pub const PARAM_IMM: u16 = PARAM_DECIMAL | PARAM_CHAR;
pub const PARAM_REG_OR_IMM: u16 = PARAM_REG | PARAM_IMM;

/// Whether the `PARAM_*` type is a register of any size.
pub fn param_is_reg(param: u16) -> bool {
    param & PARAM_REG != 0
}

/// A line starting with this (after spaces and tabs) is a comment.
pub const COMMENT_START: &[u8] = b"What the hell happened here?";
pub const OR_DRAW_25_START: &[u8] = b"or";
pub const OR_DRAW_25_END: &[u8] = b"draw 25";
/// Follows a parameter that is a pointer (`[reg]`).
pub const POINTER_SUFFIX: &[u8] = b"do you know de wey";

/// One entry of [`COMMAND_LIST`].
pub struct Command {
    pub opcode: Opcode,
    /// The text of the command; every `{p}` is a parameter. It takes up a word of its own, or
    /// a word with a `(` before it or a `:` after it.
    pub pattern: &'static str,
    /// The allowed `PARAM_*` types of each parameter.
    pub allowed_param_types: &'static [u16],
    /// Generates the command's code (for `-S` text and object code alike).
    pub code: CodeGen,
}

impl Command {
    /// The number of parameters the command takes (0 to 3).
    pub fn used_parameters(&self) -> usize {
        self.allowed_param_types.len()
    }

    /// Whether the first two parameters are the operands of one instruction (a register and a
    /// register or an immediate), so that their combination must be encodable.
    pub fn has_two_operands(&self) -> bool {
        matches!(self.allowed_param_types, [a, b, ..] if param_is_reg(*a) && b & PARAM_REG_OR_IMM != 0)
    }
}

/// Defines [`Opcode`] and [`COMMAND_LIST`] (every command at the index of its opcode).
macro_rules! commands {
    ($($(#[$doc:meta])* $op:ident => $pattern:literal, [$($ty:expr),*], $code:expr;)*) => {
        /// A command, numbered by its position in [`COMMAND_LIST`].
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(u8)]
        pub enum Opcode {
            $($(#[$doc])* $op,)*
        }

        /// Every command at the index of its opcode. The parser tries the patterns in this
        /// order and takes the first one that matches.
        pub static COMMAND_LIST: &[Command] = &[
            $(Command { opcode: Opcode::$op, pattern: $pattern, allowed_param_types: &[$($ty),*], code: $code },)*
        ];
    };
}

commands! {
    // Functions
    FuncDef => "I like to have fun, fun, fun, fun, fun, fun, fun, fun, fun, fun {p}", [PARAM_FUNC_NAME], tr::func_def;
    Return => "right back at ya, buckaroo", [], tr::ret;
    ReturnOne => "no, I don't think I will", [], tr::ret_one;
    ReturnZero => "I see this as an absolute win", [], tr::ret_zero;
    Call => "{p}: whomst has summoned the almighty one", [PARAM_FUNC_NAME], tr::call;
    // Stack operations
    Push => "stonks {p}", [PARAM_REG64 | PARAM_IMM], tr::push;
    Pop => "not stonks {p}", [PARAM_REG64], tr::pop;
    // Logical Operations
    And => "bitconneeeeeeect {p} {p}", [PARAM_REG, PARAM_REG_OR_IMM], tr::and;
    Not => "{p} \\s", [PARAM_REG], tr::not;
    // Register Manipulation
    Zero => "sneak 100 {p}", [PARAM_REG], tr::zero;
    Mov => "{p} is brilliant, but I like {p}", [PARAM_REG, PARAM_REG_OR_IMM], tr::mov;
    // Arithmetic operations
    Upvote => "upvote {p}", [PARAM_REG], tr::upvote;
    Downvote => "downvote {p}", [PARAM_REG], tr::downvote;
    Parry => "parry {p} you filthy casual {p}", [PARAM_REG_OR_IMM, PARAM_REG], tr::parry;
    Add => "{p} units are ready, with {p} more well on the way", [PARAM_REG, PARAM_REG_OR_IMM], tr::add;
    Shl => "upgrades, people. Upgrades {p}", [PARAM_REG], tr::shl;
    Shr => "they had us in the first half, not gonna lie {p}", [PARAM_REG], tr::shr;
    Imul => "{p} is getting out of hand, now there are {p} of them", [PARAM_REG64 | PARAM_REG32, PARAM_REG64 | PARAM_REG32 | PARAM_IMM], tr::imul;
    Divide => "look at what {p} needs to mimic a fraction of {p}", [PARAM_REG64 | PARAM_IMM, PARAM_REG64], tr::divide;
    Power => "{p} UNLIMITED POWER {p}", [PARAM_REG64, PARAM_REG64 | PARAM_IMM], tr::power;
    // Jumps and Jump Markers
    Upgrade => "upgrade", [], tr::upgrade_label;
    FuckGoBack => "fuck go back", [], tr::upgrade_jump;
    Banana => "banana", [], tr::banana_label;
    WhereBanana => "where banana", [], tr::banana_jump;
    Monke => "monke {p}", [PARAM_MONKE_LABEL], tr::monke_label;
    ReturnToMonke => "return to monke {p}", [PARAM_MONKE_LABEL], tr::monke_jump;
    WhoWouldWin => "who would win? {p} or {p}", [PARAM_REG, PARAM_REG_OR_IMM], tr::who_would_win;
    Wins => "{p} wins", [PARAM_REG_OR_IMM], tr::wins_label;
    /// A comparison with its own labels: `who would win? rax or rbx (2026 edition)` jumps to
    /// `2026: rax wins` or `2026: rbx wins`
    WhoWouldWinEdition => "who would win? {p} or {p} ({p} edition)", [PARAM_REG, PARAM_REG_OR_IMM, PARAM_COMPARISON_LABEL], tr::who_would_win_edition;
    EditionWins => "{p}: {p} wins", [PARAM_COMPARISON_LABEL, PARAM_REG_OR_IMM], tr::edition_wins_label;
    FindDifference => "corporate needs you to find the difference between {p} and {p}", [PARAM_REG, PARAM_REG_OR_IMM], tr::find_difference;
    SamePicture => "they're the same picture", [], tr::same_picture_label;
    // IO-Operations
    WriteChar => "what can I say except {p}", [PARAM_REG8 | PARAM_CHAR], tr::write_char;
    ReadChar => "let me in. LET ME IIIIIIIIN {p}", [PARAM_REG8], tr::read_char;
    // Memory
    FreeRealEstate => "it's free real estate {p}", [PARAM_REG64], tr::free_real_estate;
    // Random commands
    Die => "guess I'll die", [], tr::die;
    ConfusedStonks => "confused stonks", [], tr::confused_stonks;
    PerfectlyBalanced => "perfectly balanced as all things should be", [], tr::perfectly_balanced;
    Illegal => "wait, that's illegal", [], tr::illegal;
    OhNo => "oh no! anyway", [], tr::oh_no;
    Over9000 => "it's over 9000 {p}", [PARAM_REG], tr::over_9000;
    RefusesToElaborate => "refuses to elaborate and leaves", [], tr::refuses_to_elaborate;
    ShallNotPass => "you shall not pass!", [], tr::shall_not_pass;
    Houston => "Houston, we have a problem", [], tr::houston;
    Rdrand => "it's dangerous to go alone, take {p}", [PARAM_REG64 | PARAM_REG32 | PARAM_REG16], tr::rdrand;
    AirSupport => "we need air support", [], tr::air_support;
    Mainframe => "Alright. I've hacked into the mainframe and disabled their algorithms.", [], tr::mainframe;
    // Debug commands
    Trap => "it's a trap", [], tr::trap;
    //Insert commands above this one
    /// "or draw 25" at the end of a line replaces the command
    OrDraw25 => "or draw 25", [], tr::draw_25;
    /// A line that is no command
    Invalid => "", [], tr::invalid;
}

impl Opcode {
    /// The command's entry in [`COMMAND_LIST`].
    pub fn command(self) -> &'static Command {
        &COMMAND_LIST[self as usize]
    }

    /// Whether the command returns from a function (with or without setting `rax`).
    pub fn is_return(self) -> bool {
        matches!(self, Opcode::Return | Opcode::ReturnOne | Opcode::ReturnZero)
    }
}

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
    /// A character with its value and its text for gas: a character literal (`'a'`, `'\n'`,
    /// `' '`) or an ASCII code like `65`.
    /// `verbatim`: gas reads `text` as `value` (not so for `'\v'`, `'\'` and ASCII codes with
    /// leading zeros, which gas reads as octal), so the `-S` text can show it.
    Char { value: u8, text: Vec<u8>, verbatim: bool },
    MonkeLabel(Vec<u8>),
    FuncName { name: Vec<u8>, mangle: bool },
    /// The edition of a "who would win" comparison.
    ComparisonLabel(Vec<u8>),
}

impl Param {
    /// The text of the parameter: shown in diagnostics and compared by the analyser (function
    /// names with their `_` prefix on macOS). Registers give their name, characters their text
    /// for gas, everything else the text from the source.
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
            Param::Decimal { text, .. }
            | Param::Char { text, .. }
            | Param::MonkeLabel(text)
            | Param::ComparisonLabel(text) => {
                Cow::Borrowed(text)
            }
        }
    }

    /// The `PARAM_*` type (0 = not classified / invalid).
    pub fn param_type(&self) -> u16 {
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
            Param::ComparisonLabel(_) => PARAM_COMPARISON_LABEL,
        }
    }
}

/// A command of the program, normally one line of code. The parser fills it in, the analyser
/// classifies the parameters and may exclude it from translation.
#[derive(Debug, Clone)]
pub struct ParsedCommand {
    pub opcode: Opcode,
    /// `None` for each slot the command does not use.
    pub parameters: [Option<Param>; 3],
    /// The index of the parameter followed by "do you know de wey" (a pointer).
    pub pointer: Option<usize>,
    pub line_num: usize,
    /// Set to false if the command must not be translated
    pub translate: bool,
}

impl ParsedCommand {
    pub fn new(line_num: usize) -> ParsedCommand {
        ParsedCommand { opcode: Opcode::Invalid, parameters: [None, None, None], pointer: None, line_num, translate: true }
    }

    /// The text of parameter `i` (see [`Param::text`]; an unused slot reads as an empty
    /// string).
    pub fn param_text(&self, i: usize, target: Target) -> Cow<'_, [u8]> {
        match &self.parameters[i] {
            Some(p) => p.text(target),
            None => Cow::Borrowed(b""),
        }
    }

    /// Whether parameter `i` is a pointer.
    pub fn is_pointer(&self, i: usize) -> bool {
        self.pointer == Some(i)
    }

    /// The `PARAM_*` type of parameter `i`.
    pub fn param_type(&self, i: usize) -> u16 {
        self.parameters[i].as_ref().map_or(0, Param::param_type)
    }

    /// In bully mode, a pointer operand gets a (random) operand size if nothing else determines
    /// it: for commands whose other operand gives the size, only if that operand is not a
    /// register; always for the one-operand commands that need a size.
    pub fn bully_pointer_needs_size(&self, index: usize) -> bool {
        match pointer_use(self.opcode, index) {
            PointerUse::Sized => !param_is_reg(self.param_type(1 - index)),
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

/// How the generated code for `opcode` uses parameter `index` when it is a pointer.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_table() {
        for (i, command) in COMMAND_LIST.iter().enumerate() {
            assert_eq!(command.opcode as usize, i);
            // every parameter has a type, and parse_line only supports the parameter words "{p}",
            // "{p}:" and "({p}"
            assert_eq!(command.pattern.matches("{p}").count(), command.used_parameters(), "{:?}", command.pattern);
            for word in command.pattern.split(' ').filter(|w| w.contains("{p}")) {
                assert!(matches!(word, "{p}" | "{p}:" | "({p}"), "{:?}", command.pattern);
            }
        }
    }

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
