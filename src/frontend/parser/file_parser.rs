//! Port of compiler/parser/fileParser.c

use crate::frontend::commands::*;
use crate::frontend::libc::{cstr, strncmp_eq, StrTok};
use crate::frontend::log::{print_debug, print_error, Arg};

/// Reads the file line by line exactly like the C `getLine()`: a line includes its `\n`, and a
/// `\r` in the second to last position is turned into the line end (CRLF handling — the C code
/// does this on every platform). Returns the line buffer and the number of bytes read.
pub struct LineReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> LineReader<'a> {
    pub fn new(data: &'a [u8]) -> LineReader<'a> {
        LineReader { data, pos: 0 }
    }
}

impl Iterator for LineReader<'_> {
    type Item = (Vec<u8>, usize);

    fn next(&mut self) -> Option<(Vec<u8>, usize)> {
        if self.pos >= self.data.len() {
            return None;
        }
        let rest = &self.data[self.pos..];
        let len = match rest.iter().position(|&b| b == b'\n') {
            Some(p) => p + 1,
            None => rest.len(),
        };
        self.pos += len;
        let mut line = rest[..len].to_vec();
        if len >= 2 && line[len - 2] == b'\r' {
            line[len - 2] = b'\n';
            line.truncate(len - 1);
        }
        Some((line, len))
    }
}

fn remove_line_breaks_and_tabs(line: &[u8]) -> Vec<u8> {
    let mut l = cstr(line).to_vec();
    while matches!(l.last(), Some(b'\t' | b'\n' | b' ')) {
        l.pop();
    }
    l
}

fn is_line_of_interest(line: &[u8], line_length: usize) -> bool {
    let mut i = 0;
    while i < line.len() && (line[i] == b'\t' || line[i] == b' ') {
        i += 1;
    }
    let c = line.get(i).copied().unwrap_or(0);
    c != b'\n' && line_length != i && !strncmp_eq(&line[i..], COMMENT_START, COMMENT_START.len())
}

fn get_lines_of_code(data: &[u8]) -> usize {
    LineReader::new(data).filter(|(line, len)| is_line_of_interest(line, *len)).count()
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

    for (i, command) in COMMAND_LIST.iter().enumerate().take(NUMBER_OF_COMMANDS - 2) {
        let mut pattern_tok = StrTok::new(command.pattern.as_bytes());
        let mut line_tok = StrTok::new(line);
        let mut parameters: Vec<Param> = Vec::new();
        parsed.is_pointer = 0;

        let mut command_token = pattern_tok.next_token(b" \t").map(|t| t.to_vec());
        let mut line_token = line_tok.next_token(b" \t").map(|t| t.to_vec());
        let mut mismatch = false;

        while let (Some(ct), Some(lt)) = (&command_token, &line_token) {
            print_debug(log_level, "\tcomparing with %s", &[Arg::S(ct)]);

            if let Some(p_pos) = ct.windows(3).position(|w| w == b"{p}") {
                let chars_before = p_pos;
                let chars_after = ct.len() - chars_before - 3;
                let tail_matches = match lt.len().checked_sub(chars_after) {
                    Some(start) => strncmp_eq(&ct[chars_before + 3..], &lt[start..], chars_after),
                    None => false,
                };
                if strncmp_eq(ct, lt, chars_before) && tail_matches {
                    print_debug(log_level, "\t\t%s contains a parameter", &[Arg::S(lt)]);
                    let parameter_length = lt.len().saturating_sub(chars_before + chars_after);
                    // Function names (definitions and calls) get a `_` prefix on macOS
                    let mangle = i == 0 || i == 4;
                    parameters.push(Param::Raw { text: lt[..parameter_length].to_vec(), mangle });

                    // If the line after this parameter contains "do you know de wey", mark it as a pointer
                    if line_tok.rest().len() >= POINTER_SUFFIX.len()
                        && line_tok.rest().starts_with(POINTER_SUFFIX)
                    {
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
                        line_tok.save += POINTER_SUFFIX.len();
                    }
                } else {
                    print_debug(log_level, "\t\tMatching failed - chars before or after {p} mismatching, attempting to match next command", &[]);
                    mismatch = true;
                    break;
                }
            } else if ct != lt {
                print_debug(log_level, "\t\tMatching failed, attempting to match next command", &[]);
                mismatch = true;
                break;
            }

            command_token = pattern_tok.next_token(b" ").map(|t| t.to_vec());
            line_token = line_tok.next_token(b" ").map(|t| t.to_vec());
        }

        if mismatch {
            continue;
        }

        match (&command_token, &line_token) {
            (None, None) => {
                parsed.opcode = i as u8;
                for (slot, p) in parameters.into_iter().enumerate() {
                    parsed.parameters[slot] = Some(p);
                }
                return parsed;
            }
            (_, None) => {
                print_debug(log_level, "\t\tMatching failed, lineToken is NULL while commandToken is not. Attempting to match next command", &[]);
                continue;
            }
            (None, Some(lt)) => {
                if lt.as_slice() == OR_DRAW_25_START && line_tok.rest() == OR_DRAW_25_END {
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
    let loc = get_lines_of_code(data);
    print_debug(state.log_level, "The number of lines are %lu", &[Arg::U(loc as u64)]);

    if loc == 0 {
        if state.compile_mode != CompileMode::Bully {
            print_error(state, input_file_name, 0, "file does not contain any commands", 0, &[]);
        }
        return Vec::new();
    }

    let mut commands = Vec::with_capacity(loc);
    print_debug(state.log_level, "Struct array was created successfully", &[]);

    for (line_index, (line, line_length)) in LineReader::new(data).enumerate() {
        if is_line_of_interest(&line, line_length) {
            let line = remove_line_breaks_and_tabs(&line);
            print_debug(state.log_level, "Parsing line: %s", &[Arg::S(&line)]);
            commands.push(parse_line(input_file_name, line_index + 1, &line, state));
        }
    }
    commands
}
