//! Turns the text of an input file into one [`ParsedCommand`] per line of code: blank lines and
//! comments are skipped, every other line is matched against the patterns of [`COMMAND_LIST`].
//! Parameters are stored as unclassified text ([`Param::Raw`]); the analyser classifies them.
//! In bully mode, a line that is no command becomes a deterministic replacement command.

use crate::frontend::commands::*;
use crate::frontend::log::{print_debug, print_error, print_error_fmt};
use crate::frontend::state::CompileState;
use crate::target::Target;
use std::borrow::Cow;

fn is_blank(b: &u8) -> bool {
    matches!(b, b' ' | b'\t')
}

/// The lines worth parsing with their line numbers (counted from 1): every line without its
/// line end (`\n` or `\r\n`) that is neither blank (only spaces and tabs) nor a comment.
fn lines_of_interest(data: &[u8]) -> Vec<(usize, &[u8])> {
    data.split_inclusive(|&b| b == b'\n')
        .map(|l| match l {
            [line @ .., b'\r', b'\n'] | [line @ .., b'\n'] => line,
            // Deliberate quirk: a CR before the last byte of an unterminated last line ends that
            // line, and the byte after the CR is dropped
            [line @ .., b'\r', _] => line,
            _ => l,
        })
        .enumerate()
        .filter(|(_, l)| {
            let text = &l[l.iter().position(|b| !is_blank(b)).unwrap_or(l.len())..];
            !text.is_empty() && !text.starts_with(COMMENT_START)
        })
        .map(|(index, l)| (index + 1, l))
        .collect()
}

/// The text of a line that is matched against the commands: everything before the first NUL
/// byte, without trailing spaces and tabs. Leading whitespace is kept for error messages.
fn line_text(line: &[u8]) -> &[u8] {
    let line = line.split(|&b| b == 0).next().unwrap_or(line);
    &line[..line.iter().rposition(|b| !is_blank(b)).map_or(0, |p| p + 1)]
}

/// Splits the next token off `rest` like `strtok_r`: leading delimiters are skipped, the token
/// ends at the next delimiter, and that one delimiter is consumed.
fn next_token<'a>(rest: &mut &'a [u8], delims: &[u8]) -> Option<&'a [u8]> {
    let Some(start) = rest.iter().position(|b| !delims.contains(b)) else {
        *rest = &[];
        return None;
    };
    let s = &rest[start..];
    let end = s.iter().position(|b| delims.contains(b)).unwrap_or(s.len());
    *rest = s.get(end + 1..).unwrap_or(&[]);
    Some(&s[..end])
}

/// The parameter texts bully mode picks from (by `computed_index`) for a replacement command.
const RANDOM_PARAMS: [&str; 41] = [
    "rax", "rcx", "rbx", "r8", "r9", "r10", "r12", "rsp", "rbp", "ax", "al", "r8b", "r9d", "r14b",
    "99", "1238", "12", "420", "987654321", "8", "9", "69", "8268", "2", "_", "a", "b", "d", "f",
    "F", "sigreturn", "uaauuaa", "uau", "uu", "main", "gets", "srand", "mprotect", "au", "uwu",
    "space",
];

/// The result of matching a line against the pattern of one command.
enum Match {
    /// The parameters (unclassified) and the index of the pointer parameter
    Command(Vec<Param>, Option<usize>),
    /// The line matches up to a trailing "or draw 25"
    OrDraw25,
    Mismatch,
}

/// Matches `line` against the pattern of `command` word by word. Words are separated by runs of
/// spaces; the first word may also be separated by tabs.
fn match_command(command: &Command, line: &[u8], input_file_name: &[u8], line_num: usize, state: &mut CompileState) -> Match {
    let mut pattern_rest = command.pattern.as_bytes();
    let mut line_rest = line;
    let mut parameters = Vec::new();
    let mut pointer = None;
    let mut delims: &[u8] = b" \t";
    loop {
        let pattern_word = next_token(&mut pattern_rest, delims);
        let line_word = next_token(&mut line_rest, delims);
        delims = b" ";
        let (pattern_word, line_word) = match (pattern_word, line_word) {
            (Some(p), Some(l)) => (p, l),
            (None, None) => return Match::Command(parameters, pointer),
            (None, Some(l)) if l == OR_DRAW_25_START && line_rest == OR_DRAW_25_END => return Match::OrDraw25,
            _ => return Match::Mismatch,
        };

        // A parameter word of a pattern is "{p}", "{p}:" or "({p}"
        let Some(at) = pattern_word.windows(3).position(|w| w == b"{p}") else {
            if pattern_word != line_word {
                return Match::Mismatch;
            }
            continue;
        };
        let (prefix, suffix) = (&pattern_word[..at], &pattern_word[at + 3..]);
        let Some(text) = line_word.strip_prefix(prefix).and_then(|w| w.strip_suffix(suffix)) else {
            return Match::Mismatch;
        };
        // Function names (definitions and calls) get a `_` prefix on macOS
        let mangle = matches!(command.opcode, Opcode::FuncDef | Opcode::Call);
        parameters.push(Param::Raw { text: text.to_vec(), mangle });

        if let Some(after_suffix) = line_rest.strip_prefix(POINTER_SUFFIX) {
            if pointer.is_some() && !state.bully() {
                print_error(state, input_file_name, line_num, "Only one parameter is allowed to be a pointer");
            }
            pointer = Some(parameters.len() - 1);
            line_rest = after_suffix;
        }
    }
}

/// Bully mode replaces a line that is no command with a (deterministic) valid one, chosen by
/// the global `computed_index`, so it also depends on the previous invalid lines.
fn bully_replacement(parsed: &mut ParsedCommand, line: &[u8], input_file_name: &[u8], state: &mut CompileState) {
    for &b in line {
        state.computed_index = state.computed_index.wrapping_add(u64::from(b));
    }
    let first = u64::from(input_file_name.first().copied().unwrap_or(0));
    state.computed_index = (state.computed_index.wrapping_mul(parsed.line_num as u64) % 420).wrapping_mul(first);
    let ci = state.computed_index;

    // (any command but `Invalid`, also `OrDraw25`)
    let command = &COMMAND_LIST[(ci % Opcode::Invalid as u64) as usize];
    parsed.opcode = command.opcode;
    let count = RANDOM_PARAMS.len() as u64;
    let picks = [ci % count, ci.wrapping_mul(first) % count, ci.wrapping_mul(first).wrapping_mul(first) % count];
    for (slot, pick) in parsed.parameters.iter_mut().zip(picks).take(command.used_parameters()) {
        // (no macOS `_` prefix for these)
        *slot = Some(Param::Raw { text: RANDOM_PARAMS[pick as usize].as_bytes().to_vec(), mangle: false });
    }
}

/// Parses one line (as cut by [`line_text`]) as the first command in [`COMMAND_LIST`] whose
/// pattern matches it, or as `OrDraw25` if that match ends in "or draw 25". A line that matches
/// nothing is reported as invalid and stays `Invalid` (bully mode replaces it instead).
fn parse_line(input_file_name: &[u8], line_num: usize, line: &[u8], state: &mut CompileState) -> ParsedCommand {
    let mut parsed = ParsedCommand::new(line_num);
    for command in &COMMAND_LIST[..Opcode::OrDraw25 as usize] {
        match match_command(command, line, input_file_name, line_num, state) {
            Match::Mismatch => continue,
            Match::OrDraw25 => parsed.opcode = Opcode::OrDraw25,
            Match::Command(parameters, pointer) => {
                parsed.opcode = command.opcode;
                parsed.pointer = pointer;
                for (slot, p) in parsed.parameters.iter_mut().zip(parameters) {
                    *slot = Some(p);
                }
            }
        }
        print_debug(state.log_level, format!("line {line_num}: {}", describe(&parsed, state.target)));
        return parsed;
    }

    if state.bully() {
        bully_replacement(&mut parsed, line, input_file_name, state);
        print_debug(state.log_level, format!("line {line_num}: invalid, bully mode replaces it with {}", describe(&parsed, state.target)));
    } else {
        print_debug(state.log_level, format!("line {line_num}: invalid"));
        print_error_fmt(state, input_file_name, line_num, "Invalid command: \"%s\"", &[line]);
    }
    parsed
}

/// A parsed command for debug messages, e.g. `Mov ["rax" (pointer), "5"]`.
fn describe(cmd: &ParsedCommand, target: Target) -> String {
    let params: Vec<String> = (0..cmd.opcode.command().used_parameters())
        .map(|i| {
            let text = cmd.parameters[i].as_ref().map_or(Cow::Borrowed(&b""[..]), |p| p.text(target));
            let pointer = if cmd.is_pointer(i) { " (pointer)" } else { "" };
            format!("{:?}{pointer}", String::from_utf8_lossy(&text))
        })
        .collect();
    format!("{:?} [{}]", cmd.opcode, params.join(", "))
}

/// Parses an input file line by line and returns the parsed commands (one per line of interest).
pub fn parse_commands(data: &[u8], input_file_name: &[u8], state: &mut CompileState) -> Vec<ParsedCommand> {
    let lines = lines_of_interest(data);
    print_debug(state.log_level, format!("{}: {} lines of code", String::from_utf8_lossy(input_file_name), lines.len()));

    if lines.is_empty() && !state.bully() {
        print_error(state, input_file_name, 0, "file does not contain any commands");
    }
    lines.into_iter().map(|(line_num, line)| parse_line(input_file_name, line_num, line_text(line), state)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(s: &[u8]) -> Vec<&[u8]> {
        let mut rest = s;
        let mut out = Vec::new();
        let mut delims = &b" \t"[..];
        while let Some(t) = next_token(&mut rest, delims) {
            out.push(t);
            delims = b" ";
        }
        out
    }

    #[test]
    fn tokenizer() {
        assert!(tokens(b"").is_empty());
        assert!(tokens(b"  \t ").is_empty());
        assert_eq!(tokens(b"\t upvote  rax "), [&b"upvote"[..], b"rax"]);
        // only the first token is also split at tabs
        assert_eq!(tokens(b"\tupvote\trax\tx y"), [&b"upvote"[..], b"rax\tx", b"y"]);

        // exactly one delimiter is consumed after a token
        let mut rest = &b"rax  do you know de wey"[..];
        assert_eq!(next_token(&mut rest, b" "), Some(&b"rax"[..]));
        assert_eq!(rest, b" do you know de wey");
        let mut rest = &b"or draw 25"[..];
        assert_eq!(next_token(&mut rest, b" \t"), Some(&b"or"[..]));
        assert_eq!(rest, OR_DRAW_25_END);
        assert_eq!(next_token(&mut rest, b" "), Some(&b"draw"[..]));
        assert_eq!(next_token(&mut rest, b" "), Some(&b"25"[..]));
        assert_eq!(next_token(&mut rest, b" "), None);
        assert!(rest.is_empty());
    }

    fn lines(data: &[u8]) -> Vec<(usize, &[u8])> {
        lines_of_interest(data).into_iter().map(|(n, l)| (n, line_text(l))).collect()
    }

    #[test]
    fn line_splitting() {
        assert!(lines(b"").is_empty());
        assert!(lines(b"\n\r\n \t\n\t\r\n   ").is_empty());
        assert_eq!(lines(b"a\nb"), [(1, &b"a"[..]), (2, b"b")]);
        assert_eq!(lines(b"a\r\n\r\n  b \t\r\n"), [(1, &b"a"[..]), (3, b"  b")]);
        // only one CR before the LF is part of the line end; other CRs stay
        assert_eq!(lines(b"a\r\r\nb\rc\nd\r"), [(1, &b"a\r"[..]), (2, b"b\rc"), (3, b"d\r")]);
        assert_eq!(lines(b"\r\r\n  \r\n"), [(1, &b"\r"[..])]);
        // an unterminated last line with a CR before its last byte ends at that CR
        assert_eq!(lines(b"a\nb\rc"), [(1, &b"a"[..]), (2, b"b")]);
        assert!(lines(b"\r\r").is_empty());
        // tabs and spaces are trimmed at the end only; other whitespace is a command
        assert_eq!(lines(b"\tx \t\n\x0b\n\x0c \n"), [(1, &b"\tx"[..]), (2, b"\x0b"), (3, b"\x0c")]);
        // comments, also indented ones
        assert_eq!(
            lines(b"What the hell happened here?\n \tWhat the hell happened here? x\nWhat the hell\n"),
            [(3, &b"What the hell"[..])]
        );
        // a NUL byte cuts the line after the blank/comment check
        assert_eq!(
            lines(b"a b\0c\n\0x\n \t\0\nWhat the hell\0 happened here?\n"),
            [(1, &b"a b"[..]), (2, b""), (3, b""), (4, b"What the hell")]
        );
    }

    fn parse(data: &[u8], target: Target) -> (Vec<ParsedCommand>, u32) {
        let mut state = CompileState::new(target);
        let commands = parse_commands(data, b"test.memeasm", &mut state);
        (commands, state.compiler_errors)
    }

    /// The parser only produces raw parameters (the first two; see [`edition`] for the third)
    fn params(c: &ParsedCommand) -> [Option<&[u8]>; 2] {
        [0, 1].map(|i| raw(c, i))
    }

    fn edition(c: &ParsedCommand) -> Option<&[u8]> {
        raw(c, 2)
    }

    fn raw(c: &ParsedCommand, i: usize) -> Option<&[u8]> {
        c.parameters[i].as_ref().map(|p| match p {
            Param::Raw { text, .. } => &text[..],
            other => panic!("unexpected {other:?}"),
        })
    }

    #[test]
    fn commands() {
        let src = b"I like to have fun, fun, fun, fun, fun, fun, fun, fun, fun, fun main\r\n\
            \tupvote\trax  \r\n\
            What the hell happened here? upvote rax\n\
            \n\
            \tf: whomst has summoned the almighty one\n\
            \trax do you know de wey is brilliant, but I like 5\n\
            \trax is brilliant, but I like rbx do you know de wey\n\
            \tupvote rax or draw 25\n\
            \tupvote rbx\0 junk\n\
            right back at ya, buckaroo";
        let (c, errors) = parse(src, Target::Linux);
        assert_eq!(errors, 0);
        let summary: Vec<_> = c.iter().map(|c| (c.line_num, c.opcode, c.pointer, params(c))).collect();
        let (mov, upvote) = (Opcode::Mov, Opcode::Upvote);
        assert_eq!(
            summary,
            [
                (1, Opcode::FuncDef, None, [Some(&b"main"[..]), None]),
                (2, upvote, None, [Some(&b"rax"[..]), None]),
                (5, Opcode::Call, None, [Some(&b"f"[..]), None]),
                (6, mov, Some(0), [Some(&b"rax"[..]), Some(b"5")]),
                (7, mov, Some(1), [Some(&b"rax"[..]), Some(b"rbx")]),
                (8, Opcode::OrDraw25, None, [None, None]),
                (9, upvote, None, [Some(&b"rbx"[..]), None]),
                (10, Opcode::Return, None, [None, None]),
            ]
        );

        // comparisons with an edition ("({p}" parameter words) and their labels
        let src = b"who would win? rax or 5 (2026 edition)\n\
            who would win? rax or rbx\n\
            2026: rax wins\n\
            rax wins\n\
            who would win? rax or rbx (2026 edition) or draw 25\n\
            who would win? rax or rbx ( edition)\n\
            :: rax wins";
        let (c, errors) = parse(src, Target::Linux);
        assert_eq!(errors, 0);
        let summary: Vec<_> = c.iter().map(|c| (c.opcode, params(c), edition(c))).collect();
        assert_eq!(
            summary,
            [
                (Opcode::WhoWouldWinEdition, [Some(&b"rax"[..]), Some(b"5")], Some(&b"2026"[..])),
                (Opcode::WhoWouldWin, [Some(&b"rax"[..]), Some(b"rbx")], None),
                (Opcode::EditionWins, [Some(&b"2026"[..]), Some(b"rax")], None),
                (Opcode::Wins, [Some(&b"rax"[..]), None], None),
                (Opcode::OrDraw25, [None, None], None),
                (Opcode::WhoWouldWinEdition, [Some(&b"rax"[..]), Some(b"rbx")], Some(&b""[..])),
                (Opcode::EditionWins, [Some(&b":"[..]), Some(b"rax")], None),
            ]
        );
        for line in [&b"who would win? rax or rbx 2026 edition"[..], b"who would win? rax or rbx (2026 edition", b"2026 : rax wins"] {
            let (c, errors) = parse(line, Target::Linux);
            assert_eq!((c[0].opcode, errors), (Opcode::Invalid, 1), "{:?}", String::from_utf8_lossy(line));
        }

        // macOS function names get their underscore
        let (c, _) = parse(b"I like to have fun, fun, fun, fun, fun, fun, fun, fun, fun, fun main\n", Target::MacOS);
        assert_eq!(&*c[0].parameters[0].as_ref().unwrap().text(Target::MacOS), b"_main");
    }

    #[test]
    fn invalid_commands() {
        // runs of spaces separate tokens, but tabs only end the first token: later tabs are part
        // of a token (the analyser rejects such parameters)
        for (line, param) in [
            (&b"upvote  rax"[..], &b"rax"[..]),
            (b"\tupvote \trax", b"\trax"),
            (b"upvote\t\trax", b"\trax"),
            (b"upvote rax\t", b"rax"),
            (b"upvote rax\t\x0b", b"rax\t\x0b"),
        ] {
            let (c, errors) = parse(line, Target::Linux);
            let expected = (Opcode::Upvote, [Some(param), None], 0);
            assert_eq!((c[0].opcode, params(&c[0]), errors), expected);
        }

        for line in [
            &b"upvote\trax\tor draw 25"[..],
            b"f whomst has summoned the almighty one",
            b"rax is brilliant,\tbut I like 5",
            b"upvote rax or draw 25 x",
            b"upvote rax or draw  25",
            b"\0upvote rax",
        ] {
            let (c, errors) = parse(line, Target::Linux);
            assert_eq!((c[0].opcode, errors), (Opcode::Invalid, 1), "{:?}", String::from_utf8_lossy(line));
        }
        let line = b"rax do you know de wey is brilliant, but I like rbx do you know de wey";
        let (c, errors) = parse(line, Target::Linux);
        assert_eq!((c[0].pointer, errors), (Some(1), 1));

        for empty in [&b""[..], b"\n \t\r\nWhat the hell happened here?"] {
            let (c, errors) = parse(empty, Target::Linux);
            assert_eq!((c.len(), errors), (0, 1));
        }
    }
}
