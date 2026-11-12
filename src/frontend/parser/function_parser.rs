//! Splits the commands of a file into functions.

use crate::frontend::commands::*;
use crate::frontend::log::{print_debug, print_error};
use crate::frontend::state::{CompileState, Function};

/// The number of commands of the function whose definition is `commands[0]`: up to its last
/// return statement before the next function definition. A function without a return statement
/// is only its definition (the commands after it then belong to no function).
fn function_length(commands: &[ParsedCommand], input_file_name: &[u8], state: &mut CompileState) -> usize {
    let body = &commands[1..];
    let end = body.iter().position(|c| c.opcode == Opcode::FuncDef).unwrap_or(body.len());
    let last_return = body[..end].iter().rposition(|c| c.opcode.is_return());
    if !state.bully() {
        if let Some(next_definition) = body.get(end) {
            if !body[..end].last().is_some_and(|c| c.opcode.is_return()) {
                let message = "expected a return statement, but got a new function definition";
                print_error(state, input_file_name, next_definition.line_num, message);
            }
        }
        if last_return.is_none() {
            print_error(state, input_file_name, commands[0].line_num, "function does not return");
        }
    }
    last_return.map_or(1, |r| r + 2)
}

/// A function that bully mode makes up for commands outside of any function, named after the
/// number of commands up to the end of `commands`.
fn injected_function(commands: Vec<ParsedCommand>, end_index: usize) -> Function {
    let func_name = FUNCTION_NAMES[end_index % FUNCTION_NAMES.len()];
    let mut def = ParsedCommand::new(69);
    def.opcode = Opcode::FuncDef;
    def.parameters[0] = Some(Param::Raw { text: func_name.as_bytes().to_vec(), mangle: false });
    let mut ret = ParsedCommand::new(420);
    ret.opcode = Opcode::Return;
    let commands = std::iter::once(def).chain(commands).chain(std::iter::once(ret)).collect();
    Function { commands, injected: true }
}

fn lines(commands: &[ParsedCommand]) -> String {
    match commands {
        [first, .., last] => format!("lines {}-{}", first.line_num, last.line_num),
        [only] => format!("line {}", only.line_num),
        [] => "no lines".into(),
    }
}

/// Splits the file's commands into functions and orphaned commands (bully mode puts each run of
/// orphaned commands into an injected function of its own).
pub fn parse_functions(
    file_name: &[u8],
    mut commands: Vec<ParsedCommand>,
    state: &mut CompileState,
) -> (Vec<Function>, Vec<ParsedCommand>) {
    let mut functions = Vec::new();
    let mut orphans = Vec::new();
    let mut taken = 0;
    while !commands.is_empty() {
        let orphan_count = commands.iter().position(|c| c.opcode == Opcode::FuncDef).unwrap_or(commands.len());
        if orphan_count > 0 {
            let run: Vec<_> = commands.drain(..orphan_count).collect();
            taken += orphan_count;
            if state.bully() {
                let function = injected_function(run, taken);
                let name = String::from_utf8_lossy(&function.commands[0].param_text(0, state.target)).into_owned();
                print_debug(state.log_level, format!("{} belong to no function, bully mode puts them into {name}", lines(&function.commands[1..function.commands.len() - 1])));
                functions.push(function);
            } else {
                for c in &run {
                    print_error(state, file_name, c.line_num, "command does not belong to any function");
                }
                orphans.extend(run);
            }
        }
        if !commands.is_empty() {
            let length = function_length(&commands, file_name, state);
            taken += length;
            let function = Function { commands: commands.drain(..length).collect(), injected: false };
            let name = String::from_utf8_lossy(&function.commands[0].param_text(0, state.target)).into_owned();
            print_debug(state.log_level, format!("function {name}: {}", lines(&function.commands)));
            functions.push(function);
        }
    }
    (functions, orphans)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parser::file_parser::parse_commands;
    use crate::target::Target;

    /// (number of commands of each function, number of orphans, number of errors)
    fn split(src: &str) -> (Vec<usize>, usize, u32) {
        let mut state = CompileState::new(Target::Linux);
        let commands = parse_commands(src.as_bytes(), b"t.memeasm", &mut state);
        let (functions, orphans) = parse_functions(b"t.memeasm", commands, &mut state);
        (functions.iter().map(|f| f.commands.len()).collect(), orphans.len(), state.compiler_errors)
    }

    const DEF: &str = "I like to have fun, fun, fun, fun, fun, fun, fun, fun, fun, fun";

    #[test]
    fn functions() {
        let ret = "right back at ya, buckaroo";
        // a function ends at its last return statement before the next definition
        assert_eq!(split(&format!("{DEF} a\n{ret}\nupvote rax\n{ret}\n{DEF} b\n{ret}\n")), (vec![4, 2], 0, 0));
        // commands after the last return statement belong to no function
        assert_eq!(split(&format!("{DEF} a\n{ret}\nupvote rax\n{DEF} b\n{ret}\n")), (vec![2, 2], 1, 2));
        assert_eq!(split(&format!("upvote rax\n{DEF} a\n{ret}\n")), (vec![2], 1, 1));
        // without a return statement, a function is only its definition
        assert_eq!(split(&format!("{DEF} a\nupvote rax\n")), (vec![1], 1, 2));
    }

    #[test]
    fn missing_return_before_next_definition() {
        let ret = "right back at ya, buckaroo";
        // "expected a return statement" and "function does not return", also for the first
        // function of the file (the C code missed that one)
        assert_eq!(split(&format!("{DEF} a\n{DEF} b\n{ret}\n")), (vec![1, 2], 0, 2));
        assert_eq!(split(&format!("{DEF} x\n{ret}\n{DEF} a\n{DEF} b\n{ret}\n")), (vec![2, 1, 2], 0, 2));
        assert_eq!(split(&format!("{DEF} a\nupvote rax\n{DEF} b\n{ret}\n")), (vec![1, 2], 1, 3));
    }
}
