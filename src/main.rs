//! The `memeasm` executable: parses the command line, turns the options into a
//! `CompileState`, reads and parses every input file (those listed in the Breakfile if none are
//! given) and hands the result to `frontend::compile`, which analyses it and writes the
//! requested output.

mod backend;
mod frontend;
mod target;

use std::ffi::OsString;
use std::io::Read;
use std::process::exit;

use clap::Parser;

use frontend::cli::{optimisation_level, target_name, Cli};
use frontend::state::{CompileState, LogLevel, OutputMode};
use frontend::breakfile::{self, BREAKFILE_NAME};
use frontend::log::{err_str, out_str, print_debug, print_error, print_information_header, print_note};
use frontend::{bytes_to_path, compilation_failed, compile, parser, print_explanation_message};
use target::Target;

/// Prints the help page of this command. Launched by using the -h option in the terminal
fn print_help_page(program_name: &str, target: Target) {
    print_information_header(target);
    let p = program_name;
    out_str("Usage:\n");
    out_str(&format!(" {p} [options] -o outputFile [-d] inputFile...\t\tCompiles the specified file(s) into an executable\n"));
    out_str(&format!(" {p} [options] -S -o outputFile.S [-d] inputFile...\tOnly compiles the specified file(s) and saves them as x86_64 Assembly code\n"));
    out_str(&format!(" {p} [options] -O -o outputFile.o [-d] inputFile...\tOnly compiles the specified file(s) and saves them as an object file\n"));
    out_str(&format!(" {p} [options] -o outputFile [-d]\t\t\t\tCompiles the files listed in the Breakfile of the current directory\n"));
    out_str(&format!(" {p} (-h | --help)\t\t\t\t\t\tDisplays this help page\n"));
    out_str(&format!(" {p} (-v | --version)\t\t\t\t\tPrints version information\n\n"));
    out_str("Compiler options:\n");
    out_str(" -O-1, --opt-level=-1 \t- reverse optimisation stage 1: A nop is inserted after every command\n");
    out_str(" -O-2, --opt-level=-2 \t- reverse optimisation stage 2: A register is moved to and from the Stack after every command\n");
    out_str(" -O-3, --opt-level=-3 \t- reverse optimisation stage 3: A xmm-register is moved to and from the Stack using movups after every command\n");
    out_str(" -O-s, --opt-level=-s \t- reverse storage optimisation: Intentionally increases the file size by aligning end of the compiled Assembly-code to 536870912B\n");
    out_str(" -O69420, --opt-level=69420 \t- maximum optimisation. Reduces the execution to close to 0s by optimising out your entire code\n");
    out_str(" --compile-mode=<noob|bully|obfuscated> - Change the compile mode (default: noob)\n");
    out_str(" -g \t\t\t- write debug info into the compiled file. Currently, only the DWARF format is supported (Linux-only)\n");
    out_str(" --no-martyrdom \t- Disables martyrdom\n");
    out_str(" -d \t\t\t- enables debug logs\n");
    out_str(&format!(
        " --target=<linux|macos|windows> - Change the target operating system (default: {})\n",
        target_name(Target::host())
    ));
}

/// The raw bytes of a command line argument (lossily converted to UTF-8 on non-Unix hosts).
fn os_to_bytes(s: &OsString) -> Vec<u8> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        s.as_os_str().as_bytes().to_vec()
    }
    #[cfg(not(unix))]
    {
        s.to_string_lossy().into_owned().into_bytes()
    }
}

/// Reads the file `name` (described as `what` in error messages), which must be a regular file.
/// Exits with an error message if it cannot be read.
fn read_file(name: &[u8], what: &str, program_name: &[u8]) -> Vec<u8> {
    let mut file = match std::fs::File::open(bytes_to_path(name)) {
        Ok(f) => f,
        Err(e) => {
            err_str(&format!("Failed to open {what}: {}\n", frontend::log::io_error_message(&e)));
            print_explanation_message(program_name);
            exit(1);
        }
    };

    //Check that this is a regular file. If we did not check for this, an input file like "/dev/urandom" would pass without errors
    if !file.metadata().map(|m| m.is_file()).unwrap_or(false) {
        err_str(&format!("Error while opening {what}: it does not point to a regular file (e.g. it could be a directory, character device or a socket)\n"));
        print_explanation_message(program_name);
        exit(1);
    }

    let mut data = Vec::new();
    if let Err(e) = file.read_to_end(&mut data) {
        err_str(&format!("Failed to read {what}: {}\n", frontend::log::io_error_message(&e)));
        exit(1);
    }
    data
}

/// The input files listed in the Breakfile of the current directory, used when no input files
/// are given. Without a Breakfile that is an error, and so is a mistake in the Breakfile.
fn breakfile_inputs(state: &mut CompileState, program_name: &[u8]) -> Vec<Vec<u8>> {
    if !std::path::Path::new(BREAKFILE_NAME).exists() {
        err_str("Error: No input file(s) specified\n");
        print_explanation_message(program_name);
        exit(1);
    }

    print_debug(state.log_level, format!("no input files given, using the {BREAKFILE_NAME}"));
    let data = read_file(BREAKFILE_NAME.as_bytes(), BREAKFILE_NAME, program_name);
    match breakfile::parse(&data) {
        Ok(files) => files.into_iter().map(<[u8]>::to_vec).collect(),
        Err(errors) => {
            for e in errors {
                print_error(state, BREAKFILE_NAME.as_bytes(), e.line, e.message);
            }
            compilation_failed(state);
        }
    }
}

fn main() {
    let args = frontend::cli::legacy_opt_args(std::env::args_os());
    let program_name = args.first().map(os_to_bytes).unwrap_or_default();
    let program_name_str = String::from_utf8_lossy(&program_name).into_owned();

    let cli = match Cli::try_parse_from(&args) {
        Ok(cli) => cli,
        Err(e) => {
            let _ = e.print();
            exit(1);
        }
    };
    let target = cli.target.unwrap_or_else(Target::host);

    if cli.help {
        print_help_page(&program_name_str, target);
        exit(0);
    }
    if cli.version {
        //Print the version number, but without the "v" at the beginning
        out_str(&format!("{}\n", &frontend::log::VERSION_STRING[1..]));
        exit(0);
    }

    let mut state = CompileState::new(target);
    if cli.debug {
        state.log_level = LogLevel::Debug;
    }
    state.martyrdom = !cli.no_martyrdom;
    state.compile_mode = cli.compile_mode;
    state.optimisation_level = optimisation_level(cli.opt_level);
    if cli.assembly {
        state.output_mode = OutputMode::AssemblyFile;
    } else if cli.object {
        state.output_mode = OutputMode::ObjectFile;
    }
    if cli.debug_info {
        match target {
            //On Windows and MacOS, -g does not work - output a note, but don't do anything
            Target::Windows => print_note("-g cannot be used on Windows-systems, this option will be ignored.", false),
            Target::MacOS => print_note("-g cannot be used on MacOS-systems, this option will be ignored.", false),
            Target::Linux => state.use_debug_info = true,
        }
    }

    if state.use_debug_info && state.bully() {
        print_note("-g cannot be used in bully mode, this option will be ignored.", false);
        state.use_debug_info = false;
    }

    let Some(output_file) = cli.output.as_ref().map(os_to_bytes) else {
        err_str("Error: No output file specified\n");
        print_explanation_message(&program_name);
        exit(1);
    };
    let input_files: Vec<Vec<u8>> = if cli.files.is_empty() {
        breakfile_inputs(&mut state, &program_name)
    } else {
        cli.files.iter().map(os_to_bytes).collect()
    };

    //Open each file one by one and parse it
    for name in &input_files {
        let data = read_file(name, "input file", &program_name);
        print_debug(state.log_level, [&b"parsing "[..], name].concat());
        let parsed = parser::parse_file(name, &data, &mut state);
        state.files.push(parsed);
    }

    compile(state, &output_file);
}
