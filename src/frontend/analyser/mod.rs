//! Port of compiler/analyser/{analyser,functions,jumpMarkers,comparisons,randomCommands}.c

pub mod helper;
pub mod parameters;

use super::commands::*;
use super::random::Rng;
use super::log::{print_debug, print_error, print_thanos_ascii};
use helper::{check_companion_command_existence, check_duplicate_definition};

/// Runs the parameter checks on every command and then the analyses (in the C code's order:
/// by the opcode of the command each one belongs to).
pub fn analyse_commands(state: &mut CompileState) {
    let mut rng = Rng::default();
    let mut lists: Vec<Vec<CmdLoc>> = vec![Vec::new(); COMMAND_LIST.len()];

    for file in 0..state.files.len() {
        for func in 0..state.files[file].functions.len() {
            for k in 0..state.files[file].functions[func].commands.len() {
                let loc = CmdLoc { file, func, k };
                let file_name = state.files[file].file_name.clone();
                parameters::check_parameters(loc, &file_name, state);
                lists[state.cmd(loc).opcode as usize].push(loc);
            }
        }
    }
    let list = |op: Opcode| &lists[op as usize];

    analyse_functions(list(Opcode::FuncDef), state);
    if state.output_mode == OutputMode::Executable {
        // Function calls need a defined function if an executable is created
        check_companion_command_existence(list(Opcode::Call), list(Opcode::FuncDef), state, 1, false, "function");
    }
    analyse_jump_markers(Opcode::Upgrade, list(Opcode::Upgrade), list(Opcode::FuckGoBack), state);
    analyse_jump_markers(Opcode::Banana, list(Opcode::Banana), list(Opcode::WhereBanana), state);
    analyse_monke_markers(list(Opcode::Monke), list(Opcode::ReturnToMonke), state);
    analyse_who_would_win_commands(list(Opcode::WhoWouldWin), list(Opcode::Wins), state);
    analyse_theyre_the_same_picture_commands(list(Opcode::FindDifference), list(Opcode::SamePicture), state);
    set_confused_stonks_jump_label(state, &mut rng);
    choose_lines_to_be_deleted(list(Opcode::PerfectlyBalanced).len(), state, &mut rng);

    print_debug(state.log_level, "Analysis done, freeing memory");
}

/// Checks that no function name is used twice and that a main function exists for executables.
fn analyse_functions(definitions: &[CmdLoc], state: &mut CompileState) {
    check_duplicate_definition(definitions, state, false, 1, "function");

    if state.output_mode == OutputMode::Executable
        && state.compile_mode != CompileMode::Bully
        && !main_function_exists(state)
    {
        let name = state.files[0].file_name.clone();
        print_error(state, &name, 0, "unable to create an executable if no main-function was defined");
    }
}

/// Checks if a main function was defined anywhere.
pub fn main_function_exists(state: &CompileState) -> bool {
    let main_name = state.main_function_name();
    let mut functions = state.files.iter().flat_map(|f| &f.functions);
    functions.any(|f| *f.commands[0].param_text(0, state.target) == *main_name)
}

fn analyse_monke_markers(markers: &[CmdLoc], jumps: &[CmdLoc], state: &mut CompileState) {
    print_debug(state.log_level, "Beginning Monke jump label validity check");
    check_duplicate_definition(markers, state, false, 1, "monke jump marker");
    check_companion_command_existence(jumps, markers, state, 1, false, "monke jump marker");
}

fn analyse_jump_markers(opcode: Opcode, markers: &[CmdLoc], jumps: &[CmdLoc], state: &mut CompileState) {
    print_debug(state.log_level, format!("Starting jump label validity check for opcode {}", opcode as u8));
    check_duplicate_definition(markers, state, true, 0, "jump marker");
    check_companion_command_existence(jumps, markers, state, 0, true, "jump marker");
}

fn analyse_who_would_win_commands(comparisons: &[CmdLoc], labels: &[CmdLoc], state: &mut CompileState) {
    print_debug(state.log_level, "Starting analysis for \"who would win?\" command");
    check_duplicate_definition(labels, state, true, 1, "comparison jump marker");
    check_companion_command_existence(comparisons, labels, state, 2, true, "comparison jump label");
}

fn analyse_theyre_the_same_picture_commands(comparisons: &[CmdLoc], labels: &[CmdLoc], state: &mut CompileState) {
    print_debug(state.log_level, "Starting analysis for \"corporate needs you to find the difference...\" command");
    check_duplicate_definition(labels, state, true, 0, "\"they're the same picture\"");
    check_companion_command_existence(comparisons, labels, state, 0, true, "\"they're the same picture\"");
}

/// Chooses a random line of code for each input file at which the "confused stonks" jump
/// marker is placed.
fn set_confused_stonks_jump_label(state: &mut CompileState, rng: &mut Rng) {
    for i in 0..state.files.len() {
        if state.files[i].loc == 0 {
            continue;
        }
        state.files[i].random_index = rng.below(state.files[i].loc);
        print_debug(state.log_level, format!("Chose random line for jump marker: {}", state.files[i].random_index));
    }
}

/// Marks random lines as deleted depending on how often "perfectly balanced as all things
/// should be" was used.
fn choose_lines_to_be_deleted(perfectly_balanced_used: usize, state: &mut CompileState, rng: &mut Rng) {
    let ll = state.log_level;
    print_debug(ll, "Starting analysis of the \"Perfectly balanced...\" command");
    print_debug(ll, format!("\tamount of times perfectly balanced was used: {perfectly_balanced_used}"));

    let loc: usize = state.files.iter().map(|f| f.loc).sum();
    let mut lines_to_be_kept = loc;
    for _ in 0..perfectly_balanced_used {
        lines_to_be_kept /= 2;
    }
    let lines_to_be_deleted = loc - lines_to_be_kept;
    let mut lines: Vec<Vec<&mut ParsedCommand>> = state.files.iter_mut().map(File::lines_mut).collect();
    // In bully mode, lines may already be excluded from translation (e.g. duplicate jump
    // markers). The C loop then never found enough lines and hung; stop once every line is gone.
    let deletable = lines.iter().flatten().filter(|c| c.translate).count();

    print_debug(ll, format!("\tamount of lines to be deleted: {lines_to_be_deleted}"));
    if lines_to_be_deleted > 0 {
        print_thanos_ascii(lines_to_be_deleted);

        let mut selected_lines = 0;
        while selected_lines < lines_to_be_deleted.min(deletable) {
            let random_file = rng.below(lines.len());
            let file_loc = lines[random_file].len();
            if file_loc == 0 {
                // A file without code (possible in bully mode) has no line to delete; the C code
                // computed `rand() % 0` and crashed here. Draw another file instead.
                continue;
            }
            let random_line = rng.below(file_loc);
            print_debug(ll, format!("\tGenerated random line {random_line} in file {random_file}"));

            let cmd = &mut lines[random_file][random_line];
            if !cmd.translate {
                print_debug(ll, "\t\tLine was already generated");
                continue;
            }
            cmd.translate = false;
            print_debug(ll, "\t\tLine was added to the list");
            selected_lines += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parser::parse_file;
    use crate::target::Target;

    #[test]
    fn perfectly_balanced_deletes_orphaned_lines() {
        // Bully mode moves orphaned lines into an injected function; deleting such a line must
        // remove it from there (the C code deleted it from the file's array, without effect)
        let mut state = CompileState::new(Target::Linux);
        state.compile_mode = CompileMode::Bully;
        state.output_mode = OutputMode::AssemblyFile;
        let src = b"perfectly balanced as all things should be\nupvote rax\nupvote rbx\nupvote rcx\n";
        let file = parse_file(b"t.memeasm", src, &mut state);
        state.files.push(file);
        analyse_commands(&mut state);
        let functions = &state.files[0].functions;
        assert_eq!((functions.len(), functions[0].injected, functions[0].commands.len()), (1, true, 6));
        // 4 lines, half of them deleted
        assert_eq!(functions[0].commands.iter().filter(|c| !c.translate).count(), 2);
    }
}
