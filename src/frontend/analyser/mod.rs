//! Port of compiler/analyser/{analyser,functions,jumpMarkers,comparisons,randomCommands}.c

pub mod helper;
pub mod parameters;

use super::commands::*;
use super::random::Rng;
use super::log::{print_debug, print_error, print_thanos_ascii, Arg};
use helper::{check_companion_command_existence, check_duplicate_definition};

/// Runs the parameter checks on every command and then every command's analysis function.
pub fn analyse_commands(state: &mut CompileState) {
    let mut rng = Rng::default();
    let mut lists: Vec<Vec<CmdLoc>> = vec![Vec::new(); NUMBER_OF_COMMANDS];

    for file in 0..state.files.len() {
        for func in 0..state.files[file].functions.len() {
            for k in 0..state.files[file].functions[func].number_of_commands {
                let loc = CmdLoc { file, func, k };
                let file_name = state.files[file].file_name.clone();
                parameters::check_parameters(loc, &file_name, state);
                lists[state.cmd(loc).opcode as usize].push(loc);
            }
        }
    }

    for (opcode, command) in COMMAND_LIST.iter().enumerate() {
        match command.analysis {
            None => {}
            Some(Analysis::Functions) => analyse_functions(&lists, opcode, state),
            Some(Analysis::Call) => analyse_call(&lists, opcode, state),
            Some(Analysis::JumpMarkers) => analyse_jump_markers(&lists, opcode, state),
            Some(Analysis::MonkeMarkers) => analyse_monke_markers(&lists, opcode, state),
            Some(Analysis::WhoWouldWin) => analyse_who_would_win_commands(&lists, opcode, state),
            Some(Analysis::TheyreTheSamePicture) => {
                analyse_theyre_the_same_picture_commands(&lists, opcode, state)
            }
            Some(Analysis::ConfusedStonks) => set_confused_stonks_jump_label(state, &mut rng),
            Some(Analysis::PerfectlyBalanced) => choose_lines_to_be_deleted(&lists, opcode, state, &mut rng),
        }
    }

    print_debug(state.log_level, "Analysis done, freeing memory", &[]);
}

/// Checks that no function name is used twice and that a main function exists for executables.
fn analyse_functions(lists: &[Vec<CmdLoc>], opcode: usize, state: &mut CompileState) {
    check_duplicate_definition(&lists[opcode], state, false, 1, "function");

    if state.output_mode == OutputMode::Executable
        && state.compile_mode != CompileMode::Bully
        && !main_function_exists(state)
    {
        let name = state.files[0].file_name.clone();
        print_error(state, &name, 0, "unable to create an executable if no main-function was defined", 0, &[]);
    }
}

/// Checks if a main function was defined anywhere.
pub fn main_function_exists(state: &CompileState) -> bool {
    let main_name = state.main_function_name();
    (0..state.files.len()).any(|file| {
        (0..state.files[file].functions.len())
            .any(|func| *state.function_commands(file, func)[0].param_text(0, state.target) == *main_name)
    })
}

/// Function calls need a defined function if an executable is created.
fn analyse_call(lists: &[Vec<CmdLoc>], opcode: usize, state: &mut CompileState) {
    if state.output_mode == OutputMode::Executable {
        check_companion_command_existence(&lists[opcode], &lists[0], state, 1, false, "function");
    }
}

fn analyse_monke_markers(lists: &[Vec<CmdLoc>], opcode: usize, state: &mut CompileState) {
    print_debug(state.log_level, "Beginning Monke jump label validity check", &[]);
    check_duplicate_definition(&lists[opcode], state, false, 1, "monke jump marker");
    check_companion_command_existence(&lists[opcode + 1], &lists[opcode], state, 1, false, "monke jump marker");
}

fn analyse_jump_markers(lists: &[Vec<CmdLoc>], opcode: usize, state: &mut CompileState) {
    print_debug(state.log_level, "Starting jump label validity check for opcode %u", &[Arg::U(opcode as u64)]);
    check_duplicate_definition(&lists[opcode], state, true, 0, "jump marker");
    check_companion_command_existence(&lists[opcode + 1], &lists[opcode], state, 0, true, "jump marker");
}

fn analyse_who_would_win_commands(lists: &[Vec<CmdLoc>], opcode: usize, state: &mut CompileState) {
    print_debug(state.log_level, "Starting analysis for \"who would win?\" command", &[]);
    check_duplicate_definition(&lists[opcode + 1], state, true, 1, "comparison jump marker");
    check_companion_command_existence(&lists[opcode], &lists[opcode + 1], state, 2, true, "comparison jump label");
}

fn analyse_theyre_the_same_picture_commands(lists: &[Vec<CmdLoc>], opcode: usize, state: &mut CompileState) {
    print_debug(
        state.log_level,
        "Starting analysis for \"corporate needs you to find the difference...\" command",
        &[],
    );
    check_duplicate_definition(&lists[opcode + 1], state, true, 0, "\"they're the same picture\"");
    check_companion_command_existence(&lists[opcode], &lists[opcode + 1], state, 0, true, "\"they're the same picture\"");
}

/// Chooses a random line of code for each input file at which the "confused stonks" jump
/// marker is placed.
fn set_confused_stonks_jump_label(state: &mut CompileState, rng: &mut Rng) {
    for i in 0..state.files.len() {
        if state.files[i].loc == 0 {
            continue;
        }
        state.files[i].random_index = rng.below(state.files[i].loc);
        print_debug(
            state.log_level,
            "Chose random line for jump marker: %lu",
            &[Arg::U(state.files[i].random_index as u64)],
        );
    }
}

/// Marks random lines as deleted depending on how often "perfectly balanced as all things
/// should be" was used.
fn choose_lines_to_be_deleted(lists: &[Vec<CmdLoc>], opcode: usize, state: &mut CompileState, rng: &mut Rng) {
    let ll = state.log_level;
    print_debug(ll, "Starting analysis of the \"Perfectly balanced...\" command", &[]);
    let perfectly_balanced_used = lists[opcode].len();
    print_debug(
        ll,
        "\tamount of times perfectly balanced was used: %u",
        &[Arg::U(perfectly_balanced_used as u64)],
    );

    let loc: usize = state.files.iter().map(|f| f.loc).sum();
    let mut lines_to_be_kept = loc;
    for _ in 0..perfectly_balanced_used {
        lines_to_be_kept /= 2;
    }
    let lines_to_be_deleted = loc - lines_to_be_kept;
    // In bully mode, lines may already be excluded from translation (e.g. duplicate jump
    // markers). The C loop then never found enough lines and hung; stop once every line is gone.
    let deletable: usize = state
        .files
        .iter()
        .map(|f| f.parsed_commands.iter().take(f.loc).filter(|c| c.translate).count())
        .sum();

    print_debug(ll, "\tamount of lines to be deleted: %lu", &[Arg::U(lines_to_be_deleted as u64)]);
    if lines_to_be_deleted > 0 {
        print_thanos_ascii(lines_to_be_deleted);

        let file_count = state.files.len();
        let mut selected_lines = 0;
        while selected_lines < lines_to_be_deleted.min(deletable) {
            let random_file = rng.below(file_count);
            let file_loc = state.files[random_file].loc;
            if file_loc == 0 {
                // A file without code (possible in bully mode) has no line to delete; the C code
                // computed `rand() % 0` and crashed here. Draw another file instead.
                continue;
            }
            let random_line = rng.below(file_loc);
            print_debug(
                ll,
                "\tGenerated random line %lu in file %u",
                &[Arg::U(random_line as u64), Arg::U(random_file as u64)],
            );

            let cmd = &mut state.files[random_file].parsed_commands[random_line];
            if !cmd.translate {
                print_debug(ll, "\t\tLine was already generated", &[]);
                continue;
            }
            cmd.translate = false;
            print_debug(ll, "\t\tLine was added to the list", &[]);
            selected_lines += 1;
        }
    }
}
