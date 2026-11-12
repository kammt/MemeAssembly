//! Port of compiler/parser/functionParser.c

use crate::frontend::commands::*;
use crate::frontend::log::{print_debug, print_error, Arg};

fn command_type(cmd: &ParsedCommand) -> u8 {
    cmd.opcode.command().command_type
}

/// Creates a function by starting at the function definition and then traversing the command
/// array until a return statement, new function definition or end of array is found.
/// Returns the number of commands of the function.
fn parse_function(
    commands: &[ParsedCommand],
    input_file_name: &[u8],
    function_start_at_index: usize,
    state: &mut CompileState,
) -> usize {
    let function_start = &commands[function_start_at_index];
    let start_line = function_start.line_num;

    print_debug(state.log_level, "\tParsing function:", &[]);

    let mut index = 1;
    // Points to the last found return-statement and is 0 if none was found until now
    let mut function_end_index = 0;

    while function_start_at_index + index < commands.len() {
        let parsed = &commands[function_start_at_index + index];
        let ty = command_type(parsed);

        if ty == COMMAND_TYPE_FUNC_DEF {
            if function_end_index != function_start_at_index + index - 1
                && state.compile_mode != CompileMode::Bully
            {
                print_error(
                    state,
                    input_file_name,
                    parsed.line_num as u64,
                    "expected a return statement, but got a new function definition",
                    0,
                    &[],
                );
            }
            break;
        } else if ty == COMMAND_TYPE_FUNC_RETURN {
            function_end_index = function_start_at_index + index;
        }
        index += 1;
    }
    print_debug(state.log_level, "\t\tIteration stopped at index %lu", &[Arg::U(index as u64)]);

    if function_end_index == 0 && state.compile_mode != CompileMode::Bully {
        print_error(state, input_file_name, start_line as u64, "function does not return", 0, &[]);
    }

    if function_end_index != 0 {
        function_end_index - function_start_at_index + 1
    } else {
        1
    }
}

/// Splits the file's commands into functions and orphaned commands (bully mode puts each run of
/// orphaned commands into an injected function of its own).
pub fn parse_functions(
    file_name: &[u8],
    commands: Vec<ParsedCommand>,
    state: &mut CompileState,
) -> (Vec<Function>, Vec<ParsedCommand>) {
    let function_definitions =
        commands.iter().filter(|c| command_type(c) == COMMAND_TYPE_FUNC_DEF).count();
    print_debug(state.log_level, "Number of functions: %lu", &[Arg::U(function_definitions as u64)]);

    // The file is a sequence of runs: (is a function, number of commands)
    let mut runs = Vec::new();
    let mut index = 0;
    while index < commands.len() {
        let start_index = index;
        while index < commands.len() && command_type(&commands[index]) != COMMAND_TYPE_FUNC_DEF {
            if state.compile_mode != CompileMode::Bully {
                let line = commands[index].line_num as u64;
                print_error(state, file_name, line, "command does not belong to any function", 0, &[]);
            }
            index += 1;
        }
        if index > start_index {
            runs.push((false, index - start_index));
        }
        if index < commands.len() {
            let number_of_commands = parse_function(&commands, file_name, index, state);
            runs.push((true, number_of_commands));
            index += number_of_commands;
        }
    }

    let mut functions = Vec::with_capacity(function_definitions);
    let mut orphans = Vec::new();
    let mut commands = commands.into_iter();
    let mut end_index = 0;
    for (is_function, length) in runs {
        let run = commands.by_ref().take(length);
        end_index += length;
        if is_function {
            functions.push(Function { commands: run.collect(), injected: false });
        } else if state.compile_mode == CompileMode::Bully {
            // A function named after the index of the command that follows
            let func_name = FUNCTION_NAMES[end_index % FUNCTION_NAMES.len()];
            let mut def = ParsedCommand::new(69);
            def.opcode = Opcode::FuncDef;
            def.parameters[0] = Some(Param::Raw { text: func_name.as_bytes().to_vec(), mangle: false });
            let mut ret = ParsedCommand::new(420);
            ret.opcode = Opcode::Return;
            let commands = std::iter::once(def).chain(run).chain(std::iter::once(ret)).collect();
            functions.push(Function { commands, injected: true });
        } else {
            orphans.extend(run);
        }
    }
    (functions, orphans)
}
