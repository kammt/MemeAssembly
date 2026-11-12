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

/// Splits the file's commands into functions (injecting functions for orphaned commands in
/// bully mode).
pub fn parse_functions(file: &mut File, state: &mut CompileState) {
    let commands = &file.parsed_commands;
    let function_definitions =
        commands.iter().filter(|c| command_type(c) == COMMAND_TYPE_FUNC_DEF).count();
    print_debug(state.log_level, "Number of functions: %lu", &[Arg::U(function_definitions as u64)]);

    let mut functions = Vec::with_capacity(function_definitions);
    let mut command_array_index = 0;
    while command_array_index < commands.len() {
        let start_index = command_array_index;
        let mut orphaned_commands = false;
        while command_array_index < commands.len() {
            if command_type(&commands[command_array_index]) != COMMAND_TYPE_FUNC_DEF {
                orphaned_commands = true;
                if state.compile_mode != CompileMode::Bully {
                    print_error(
                        state,
                        &file.file_name,
                        commands[command_array_index].line_num as u64,
                        "command does not belong to any function",
                        0,
                        &[],
                    );
                }
                command_array_index += 1;
            } else {
                break;
            }
        }

        // In bully mode, orphaned commands (startIndex..commandArrayIndex) get a function of their own
        if state.compile_mode == CompileMode::Bully && orphaned_commands {
            let func_name = FUNCTION_NAMES[command_array_index % FUNCTION_NAMES.len()];
            let mut owned = Vec::with_capacity(command_array_index - start_index + 2);

            let mut def = ParsedCommand::new(69);
            def.opcode = Opcode::FuncDef;
            def.parameters[0] = Some(Param::Raw { text: func_name.as_bytes().to_vec(), mangle: false });
            owned.push(def);

            owned.extend_from_slice(&commands[start_index..command_array_index]);

            let mut ret = ParsedCommand::new(420);
            ret.opcode = Opcode::Return;
            owned.push(ret);

            functions.push(Function {
                number_of_commands: owned.len(),
                commands: FunctionCommands::Owned(owned),
            });
        }

        if command_array_index >= commands.len() {
            break;
        }

        let number_of_commands =
            parse_function(commands, &file.file_name, command_array_index, state);
        functions.push(Function {
            number_of_commands,
            commands: FunctionCommands::InFile(command_array_index),
        });
        command_array_index += number_of_commands;
    }

    file.functions = functions;
}
