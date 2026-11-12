//! The two checks the analyses in [`super`] are built from: duplicate definitions (of
//! functions, jump markers and comparison labels) and definitions missing for calls and jumps.

use crate::frontend::commands::ParsedCommand;
use crate::frontend::state::{CmdLoc, CompileState};
use crate::frontend::log::{format_c, print_debug, print_error_fmt, print_error_with_key, ErrorKey};

/// Whether parameter `i` of `a` and parameter `j` of `b` have the same text
/// ([`ParsedCommand::param_text`]). Labels match by text, not by value: `7` and `07` differ.
fn param_eq(state: &CompileState, a: &ParsedCommand, i: usize, b: &ParsedCommand, j: usize) -> bool {
    a.param_text(i, state.target) == b.param_text(j, state.target)
}

/// Checks `list` for duplicate definitions and reports each later one. Two definitions are
/// duplicates if their first `parameters_to_check` parameters (0 to 2) have the same text and,
/// with `once_per_file`, they are in the same file. Bully mode excludes the later one from
/// translation instead.
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
                    // The obfuscated-mode key uses an argument count of 2 although the format
                    // has 3 arguments. The key only selects which fake message obfuscated mode
                    // prints; a different count would select a different one.
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
/// (e.g. "banana") exists, in the same file if `same_file`. With `parameters_to_check` = 0 any
/// companion will do; otherwise each of the parent's first `parameters_to_check` parameters needs
/// a companion whose first parameter has its text. A companion counts for the parent's first
/// parameter before its second, so a parent with two equal parameters always lacks one for the
/// second. Bully mode excludes the parent from translation instead of reporting it.
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

