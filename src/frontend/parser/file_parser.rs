//! Port of compiler/parser/fileParser.c

use crate::frontend::commands::*;
use crate::frontend::libc::cstr;
use crate::frontend::log::{print_debug, print_error, Arg};

fn is_blank(b: &u8) -> bool {
    matches!(b, b' ' | b'\t')
}

/// The lines worth parsing with their line numbers (counted from 1): every line without its
/// line end (`\n` or `\r\n`) that is neither blank (only spaces and tabs) nor a comment.
fn lines_of_interest(data: &[u8]) -> Vec<(usize, &[u8])> {
    data.split_inclusive(|&b| b == b'\n')
        .map(|l| match l {
            [line @ .., b'\r', b'\n'] | [line @ .., b'\n'] => line,
            // Kept from the C getLine(): a CR before the last byte of an unterminated last line
            // ends that line, the byte after it is dropped
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
    let line = cstr(line);
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

const RANDOM_PARAMS: [&str; 41] = [
    "rax", "rcx", "rbx", "r8", "r9", "r10", "r12", "rsp", "rbp", "ax", "al", "r8b", "r9d", "r14b",
    "99", "1238", "12", "420", "987654321", "8", "9", "69", "8268", "2", "_", "a", "b", "d", "f",
    "F", "sigreturn", "uaauuaa", "uau", "uu", "main", "gets", "srand", "mprotect", "au", "uwu",
    "space",
];

fn parse_line(
    input_file_name: &[u8],
    line_num: usize,
    line: &[u8],
    state: &mut CompileState,
) -> ParsedCommand {
    let mut parsed = ParsedCommand::new(line_num);
    let log_level = state.log_level;

    'commands: for (i, command) in COMMAND_LIST.iter().enumerate().take(NUMBER_OF_COMMANDS - 2) {
        let mut pattern_rest = command.pattern.as_bytes();
        let mut line_rest = line;
        let mut parameters: Vec<Param> = Vec::new();
        parsed.is_pointer = 0;

        // Only the first token may also be separated by tabs
        let mut command_token = next_token(&mut pattern_rest, b" \t");
        let mut line_token = next_token(&mut line_rest, b" \t");

        while let (Some(ct), Some(lt)) = (command_token, line_token) {
            print_debug(log_level, "\tcomparing with %s", &[Arg::S(ct)]);

            // A parameter token of a pattern is "{p}" or "{p}:"
            if let Some(suffix) = ct.strip_prefix(b"{p}") {
                if lt.ends_with(suffix) {
                    print_debug(log_level, "\t\t%s contains a parameter", &[Arg::S(lt)]);
                    let parameter_length = lt.len() - suffix.len();
                    // Function names (definitions and calls) get a `_` prefix on macOS
                    let mangle = i == 0 || i == 4;
                    parameters.push(Param::Raw { text: lt[..parameter_length].to_vec(), mangle });

                    // If the line after this parameter contains "do you know de wey", mark it as a pointer
                    if let Some(after_suffix) = line_rest.strip_prefix(POINTER_SUFFIX) {
                        print_debug(
                            log_level,
                            "\t\t\t'do you know de wey' was found, interpreting as pointer",
                            &[],
                        );
                        if parsed.is_pointer != 0 && state.compile_mode != CompileMode::Bully {
                            print_error(
                                state,
                                input_file_name,
                                line_num as u64,
                                "Only one parameter is allowed to be a pointer",
                                0,
                                &[],
                            );
                        }
                        parsed.is_pointer = parameters.len() as u8;
                        line_rest = after_suffix;
                    }
                } else {
                    print_debug(log_level, "\t\tMatching failed - chars before or after {p} mismatching, attempting to match next command", &[]);
                    continue 'commands;
                }
            } else if ct != lt {
                print_debug(log_level, "\t\tMatching failed, attempting to match next command", &[]);
                continue 'commands;
            }

            command_token = next_token(&mut pattern_rest, b" ");
            line_token = next_token(&mut line_rest, b" ");
        }

        match (command_token, line_token) {
            (None, None) => {
                parsed.opcode = i as u8;
                for (slot, p) in parameters.into_iter().enumerate() {
                    parsed.parameters[slot] = Some(p);
                }
                return parsed;
            }
            (Some(_), None) => {
                print_debug(log_level, "\t\tMatching failed, lineToken is NULL while commandToken is not. Attempting to match next command", &[]);
            }
            (None, Some(lt)) => {
                if lt == OR_DRAW_25_START && line_rest == OR_DRAW_25_END {
                    print_debug(log_level, "\t\t'or draw 25' was found, replacing opcode", &[]);
                    parsed.opcode = OR_DRAW_25_OPCODE;
                    return parsed;
                }
            }
            (Some(_), Some(_)) => unreachable!(),
        }
    }

    if state.compile_mode == CompileMode::Bully {
        // In bully mode, we replace this non-working command with a (deterministic) valid one.
        // "computedIndex" is global, so the value depends on the previous illegal commands.
        for &b in line {
            state.computed_index = state.computed_index.wrapping_add(u64::from(b));
        }
        let first = u64::from(input_file_name.first().copied().unwrap_or(0));
        state.computed_index =
            (state.computed_index.wrapping_mul(line_num as u64) % 420).wrapping_mul(first);
        let ci = state.computed_index;

        parsed.opcode = (ci % (NUMBER_OF_COMMANDS as u64 - 1)) as u8;
        let used = COMMAND_LIST[parsed.opcode as usize].used_parameters;
        let count = RANDOM_PARAMS.len() as u64;
        // (no macOS `_` prefix for these)
        let random_param = |i: u64| Some(Param::Raw { text: RANDOM_PARAMS[i as usize].as_bytes().to_vec(), mangle: false });
        if used > 0 {
            parsed.parameters[0] = random_param(ci % count);
        }
        if used > 1 {
            parsed.parameters[1] = random_param(ci.wrapping_mul(first) % count);
        }
    } else {
        parsed.opcode = INVALID_COMMAND_OPCODE;
        print_error(state, input_file_name, line_num as u64, "Invalid command: \"%s\"", 1, &[Arg::S(line)]);
    }
    parsed
}

/// Parses an input file line by line and returns the parsed commands (one per line of interest).
pub fn parse_commands(data: &[u8], input_file_name: &[u8], state: &mut CompileState) -> Vec<ParsedCommand> {
    let lines = lines_of_interest(data);
    print_debug(state.log_level, "The number of lines are %lu", &[Arg::U(lines.len() as u64)]);

    if lines.is_empty() {
        if state.compile_mode != CompileMode::Bully {
            print_error(state, input_file_name, 0, "file does not contain any commands", 0, &[]);
        }
        return Vec::new();
    }
    print_debug(state.log_level, "Struct array was created successfully", &[]);

    lines
        .into_iter()
        .map(|(line_num, line)| {
            let line = line_text(line);
            print_debug(state.log_level, "Parsing line: %s", &[Arg::S(line)]);
            parse_line(input_file_name, line_num, line, state)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::target::Target;

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

    #[test]
    fn parameter_tokens_are_prefixes() {
        // parse_line only supports "{p}" at the start of a pattern token
        for command in COMMAND_LIST.iter() {
            for token in command.pattern.split(' ').filter(|t| t.contains("{p}")) {
                assert!(token == "{p}" || token == "{p}:", "{:?}", command.pattern);
            }
        }
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
        // the C getLine() quirk for an unterminated last line with a CR before its last byte
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

    fn opcode_of(pattern: &str) -> u8 {
        COMMAND_LIST.iter().position(|c| c.pattern == pattern).unwrap() as u8
    }

    fn parse(data: &[u8], target: Target) -> (Vec<ParsedCommand>, u32) {
        let mut state = CompileState::new(target);
        let commands = parse_commands(data, b"test.memeasm", &mut state);
        (commands, state.compiler_errors)
    }

    /// The parser only produces raw parameters
    fn params(c: &ParsedCommand) -> [Option<&[u8]>; 2] {
        c.parameters.each_ref().map(|p| {
            p.as_ref().map(|p| match p {
                Param::Raw { text, .. } => &text[..],
                other => panic!("unexpected {other:?}"),
            })
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
        let summary: Vec<_> = c.iter().map(|c| (c.line_num, c.opcode, c.is_pointer, params(c))).collect();
        let mov = opcode_of("{p} is brilliant, but I like {p}");
        let upvote = opcode_of("upvote {p}");
        assert_eq!(
            summary,
            [
                (1, 0, 0, [Some(&b"main"[..]), None]),
                (2, upvote, 0, [Some(&b"rax"[..]), None]),
                (5, opcode_of("{p}: whomst has summoned the almighty one"), 0, [Some(&b"f"[..]), None]),
                (6, mov, 1, [Some(&b"rax"[..]), Some(b"5")]),
                (7, mov, 2, [Some(&b"rax"[..]), Some(b"rbx")]),
                (8, OR_DRAW_25_OPCODE, 0, [None, None]),
                (9, upvote, 0, [Some(&b"rbx"[..]), None]),
                (10, opcode_of("right back at ya, buckaroo"), 0, [None, None]),
            ]
        );

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
            let expected = (opcode_of("upvote {p}"), [Some(param), None], 0);
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
            assert_eq!((c[0].opcode, errors), (INVALID_COMMAND_OPCODE, 1), "{:?}", String::from_utf8_lossy(line));
        }
        let line = b"rax do you know de wey is brilliant, but I like rbx do you know de wey";
        let (c, errors) = parse(line, Target::Linux);
        assert_eq!((c[0].is_pointer, errors), (2, 1));

        for empty in [&b""[..], b"\n \t\r\nWhat the hell happened here?"] {
            let (c, errors) = parse(empty, Target::Linux);
            assert_eq!((c.len(), errors), (0, 1));
        }
    }
}
