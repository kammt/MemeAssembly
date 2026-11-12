//! Port of compiler/analyser/analysisHelper.c

use crate::frontend::commands::{CmdLoc, CompileMode, CompileState, ParsedCommand};
use crate::frontend::libc::cstr;
use crate::frontend::log::{print_debug, print_error, Arg};

/// What the C code's `printf("%s", parameters[0])` shows for a slot it never initialised.
fn param_for_debug(state: &CompileState, l: CmdLoc) -> Vec<u8> {
    match &state.cmd(l).parameters[0] {
        Some(p) => p.text(state.target).into_owned(),
        None => b"(null)".to_vec(),
    }
}

/// Labels match by their text, as in the C code.
fn param_eq(state: &CompileState, a: &ParsedCommand, i: usize, b: &ParsedCommand, j: usize) -> bool {
    cstr(&a.param_text(i, state.target)) == cstr(&b.param_text(j, state.target))
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
            "\tLabel duplicity check for %s in line %lu in file %u",
            &[Arg::from(item_name), Arg::U(line_num as u64), Arg::U(item.file as u64)],
        );

        for &dup in &list[idx + 1..] {
            let p = param_for_debug(state, dup);
            print_debug(state.log_level, "\t\tComparing against parameter %s", &[Arg::S(&p)]);
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
                    print_error(
                        state,
                        &dup_file,
                        dup_line as u64,
                        "%s defined twice (already defined in %s:%lu)",
                        2,
                        &[Arg::from(item_name), Arg::S(&item_file), Arg::U(line_num as u64)],
                    );
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
            "\tLooking for %s of parent command in line %lu in file %u",
            &[Arg::from(item_name), Arg::U(command.line_num as u64), Arg::U(parent.file as u64)],
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
        let line = command.line_num as u64;
        if parameters_to_check == 0 {
            if !child_found[0] {
                if state.compile_mode != CompileMode::Bully {
                    print_error(state, &file_name, line, "%s was not defined", 1, &[Arg::from(item_name)]);
                } else {
                    state.cmd_mut(parent).translate = false;
                }
            }
        } else {
            if !child_found[0] {
                if state.compile_mode != CompileMode::Bully {
                    print_error(
                        state,
                        &file_name,
                        line,
                        "%s was not defined for parameter \"%s\"",
                        2,
                        &[Arg::from(item_name), Arg::S(&command.param_text(0, state.target))],
                    );
                } else {
                    state.cmd_mut(parent).translate = false;
                }
            }
            if parameters_to_check == 2 && !child_found[1] {
                if state.compile_mode != CompileMode::Bully {
                    print_error(
                        state,
                        &file_name,
                        line,
                        "%s was not defined for parameter \"%s\"",
                        2,
                        &[Arg::from(item_name), Arg::S(&command.param_text(1, state.target))],
                    );
                } else {
                    state.cmd_mut(parent).translate = false;
                }
            }
        }
    }
}
