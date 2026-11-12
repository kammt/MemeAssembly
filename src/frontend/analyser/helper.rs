//! Port of compiler/analyser/analysisHelper.c

use crate::frontend::commands::ParsedCommand;
use crate::frontend::state::{CmdLoc, CompileState};
use crate::frontend::log::{format_c, print_debug, print_error_fmt, print_error_with_key, ErrorKey};

/// Labels match by their text, as in the C code.
fn param_eq(state: &CompileState, a: &ParsedCommand, i: usize, b: &ParsedCommand, j: usize) -> bool {
    a.param_text(i, state.target) == b.param_text(j, state.target)
}

/// Checks for duplicate definitions of commands and prints an error if there is one.
pub fn check_duplicate_definition(
    list: &[CmdLoc],
    state: &mut CompileState,
    once_per_file: bool,
    parameters_to_check: u8,
    item_name: &str,
) {
    let parameters_to_check = parameters_to_check.min(2);
    for (idx, &item) in list.iter().enumerate() {
        let line_num = state.cmd(item).line_num;
        print_debug(state.log_level, format!("checking for duplicates of the {item_name} in line {line_num} of file {}", item.file));

        for &dup in &list[idx + 1..] {
            let command = state.cmd(item);
            let duplicate = state.cmd(dup);
            if (!once_per_file || dup.file == item.file)
                && (parameters_to_check < 1 || param_eq(state, command, 0, duplicate, 0))
                && (parameters_to_check != 2 || param_eq(state, command, 1, duplicate, 1))
            {
                if !state.bully() {
                    let dup_file = state.files[dup.file].file_name.clone();
                    let item_file = state.files[item.file].file_name.clone();
                    let dup_line = duplicate.line_num;
                    let format = "%s defined twice (already defined in %s:%lu)";
                    let message = format_c(format, &[item_name.as_bytes(), &item_file, line_num.to_string().as_bytes()]);
                    // (the C code passed 2 as the argument count)
                    print_error_with_key(state, &dup_file, dup_line, ErrorKey::new(format, 2), &message);
                } else {
                    // In bully mode, only the first definition is valid
                    state.cmd_mut(dup).translate = false;
                }
            }
        }
    }
}

/// Checks that for every parent command (e.g. "where banana") a companion command
/// (e.g. "banana") exists.
pub fn check_companion_command_existence(
    parents: &[CmdLoc],
    children: &[CmdLoc],
    state: &mut CompileState,
    parameters_to_check: u8,
    same_file: bool,
    item_name: &str,
) {
    let parameters_to_check = parameters_to_check.min(2);
    for &parent in parents {
        let mut child_found = [false; 2];
        let command = state.cmd(parent).clone();

        print_debug(
            state.log_level,
            format!("looking for the {item_name} of line {} in file {}", command.line_num, parent.file),
        );

        for &child in children {
            if !same_file || parent.file == child.file {
                let child_cmd = state.cmd(child);
                if parameters_to_check == 0 || param_eq(state, &command, 0, child_cmd, 0) {
                    child_found[0] = true;
                } else if parameters_to_check == 2 && param_eq(state, &command, 1, child_cmd, 0) {
                    child_found[1] = true;
                }
            }
        }

        let file_name = state.files[parent.file].file_name.clone();
        let line = command.line_num;
        if parameters_to_check == 0 {
            if !child_found[0] {
                if !state.bully() {
                    print_error_fmt(state, &file_name, line, "%s was not defined", &[item_name.as_bytes()]);
                } else {
                    state.cmd_mut(parent).translate = false;
                }
            }
        } else {
            for (i, &found) in child_found.iter().enumerate().take(parameters_to_check as usize) {
                if !found {
                    if !state.bully() {
                        let parameter = command.param_text(i, state.target);
                        let format = "%s was not defined for parameter \"%s\"";
                        print_error_fmt(state, &file_name, line, format, &[item_name.as_bytes(), &parameter]);
                    } else {
                        state.cmd_mut(parent).translate = false;
                    }
                }
            }
        }
    }
}
