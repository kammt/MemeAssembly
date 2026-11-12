//! Port of compiler/analyser/analysisHelper.c

use crate::frontend::commands::{CmdLoc, CompileMode, CompileState, ParsedCommand};
use crate::frontend::log::{print_debug, print_error_with_key, ErrorKey};

/// What the C code's `printf("%s", parameters[0])` shows for a slot it never initialised.
fn param_for_debug(state: &CompileState, l: CmdLoc) -> Vec<u8> {
    match &state.cmd(l).parameters[0] {
        Some(p) => p.text(state.target).into_owned(),
        None => b"(null)".to_vec(),
    }
}

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
        print_debug(
            state.log_level,
            format!("\tLabel duplicity check for {item_name} in line {line_num} in file {}", item.file),
        );

        for &dup in &list[idx + 1..] {
            let p = param_for_debug(state, dup);
            print_debug(state.log_level, [&b"\t\tComparing against parameter "[..], &p].concat());
            let command = state.cmd(item);
            let duplicate = state.cmd(dup);
            if (!once_per_file || dup.file == item.file)
                && (parameters_to_check < 1 || param_eq(state, command, 0, duplicate, 0))
                && (parameters_to_check != 2 || param_eq(state, command, 1, duplicate, 1))
            {
                if state.compile_mode != CompileMode::Bully {
                    let dup_file = state.files[dup.file].file_name.clone();
                    let item_file = state.files[item.file].file_name.clone();
                    let dup_line = duplicate.line_num;
                    let message = [
                        format!("{item_name} defined twice (already defined in ").as_bytes(),
                        &item_file,
                        format!(":{line_num})").as_bytes(),
                    ]
                    .concat();
                    let key = ErrorKey::new("%s defined twice (already defined in %s:%lu)", 2);
                    print_error_with_key(state, &dup_file, dup_line, key, &message);
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
            format!("\tLooking for {item_name} of parent command in line {} in file {}", command.line_num, parent.file),
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
                if state.compile_mode != CompileMode::Bully {
                    let message = format!("{item_name} was not defined");
                    let key = ErrorKey::new("%s was not defined", 1);
                    print_error_with_key(state, &file_name, line, key, message.as_bytes());
                } else {
                    state.cmd_mut(parent).translate = false;
                }
            }
        } else {
            for (i, &found) in child_found.iter().enumerate().take(parameters_to_check as usize) {
                if !found {
                    if state.compile_mode != CompileMode::Bully {
                        let parameter = command.param_text(i, state.target);
                        let message =
                            [format!("{item_name} was not defined for parameter \"").as_bytes(), &parameter, b"\""].concat();
                        let key = ErrorKey::new("%s was not defined for parameter \"%s\"", 2);
                        print_error_with_key(state, &file_name, line, key, &message);
                    } else {
                        state.cmd_mut(parent).translate = false;
                    }
                }
            }
        }
    }
}
