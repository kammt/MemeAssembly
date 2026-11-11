//! The MemeAssembly compiler (port of compiler/memeasm.c).

mod backend;
mod frontend;
mod target;

use std::ffi::OsString;
use std::io::Read;

use clap::Parser;

use frontend::cli::{events, normalize, Cli, Event};
use frontend::commands::{CompileMode, CompileState, LogLevel, OutputMode, OPT_S};
use frontend::libc::{erange_message, strtol};
use frontend::log::{err, err_str, exit, out_str, print_debug, print_information_header, print_note, Arg};
use frontend::{bytes_to_path, compile, parser, print_explanation_message};
use target::Target;

/// Prints the help page of this command. Launched by using the -h option in the terminal
fn print_help_page(program_name: &str, target: Target) {
    print_information_header(target);
    let p = program_name;
    out_str("Usage:\n");
    // The C help page still advertised `[-i | -d]`, although -i was removed (commit e99f0b3).
    out_str(&format!(" {p} [options] -o outputFile [-d] inputFile\t\tCompiles the specified file into an executable\n"));
    out_str(&format!(" {p} [options] -S -o outputFile.S [-d] inputFile\tOnly compiles the specified file and saves it as x86_64 Assembly code\n"));
    out_str(&format!(" {p} [options] -O -o outputFile.o [-d] inputFile\tOnly compiles the specified file and saves it an object file\n"));
    out_str(&format!(" {p} (-h | --help)\t\t\t\t\tDisplays this help page\n"));
    out_str(&format!(" {p} -v\t\t\t\t\t\t\tPrints version information\n\n"));
    out_str("Compiler options:\n");
    out_str(" -O-1 \t\t- reverse optimisation stage 1: A nop is inserted after every command\n");
    out_str(" -O-2 \t\t- reverse optimisation stage 2: A register is moved to and from the Stack after every command\n");
    out_str(" -O-3 \t\t- reverse optimisation stage 3: A xmm-register is moved to and from the Stack using movups after every command\n");
    out_str(" -O-s \t\t- reverse storage optimisation: Intentionally increases the file size by aligning end of the compiled Assembly-code to 536870912B\n");
    out_str(" -O69420 \t- maximum optimisation. Reduces the execution to close to 0s by optimising out your entire code\n");
    out_str(" -fcompile-mode - Change the compile mode to noob (default), bully, or obfuscated\n");
    out_str(" -g \t\t- write debug info into the compiled file. Currently, only the DWARF format is supported (Linux-only)\n");
    out_str(" -fno-martyrdom - Disables martyrdom\n");
    out_str(" -d \t\t- enables debug logs\n");
    let host = match Target::host() {
        Target::Linux => "linux",
        Target::MacOS => "macos",
        Target::Windows => "windows",
    };
    out_str(&format!(" -ftarget=<linux|macos|windows> - Change the target operating system (default: {host})\n"));
}

/// `-O<level>`: `Ok(level)` or the error message of memeasm.c.
fn parse_optimisation_level(level: &str) -> Result<i64, String> {
    if level == "-s" {
        return Ok(OPT_S);
    }
    let r = strtol(level.as_bytes());
    if r.overflow {
        return Err(format!("Invalid optimisation level specified: {}\n", erange_message()));
    }
    if r.end == 0 || r.end != level.len() || ![69420, -1, -2, -3].contains(&r.value) {
        return Err(format!("Invalid optimisation level specified: {level}\n"));
    }
    Ok(r.value)
}

fn parse_compile_mode(mode: &str) -> Option<CompileMode> {
    match mode {
        "bully" => Some(CompileMode::Bully),
        "obfuscated" => Some(CompileMode::Obfuscated),
        "noob" => Some(CompileMode::Noob),
        _ => None,
    }
}

const INVALID_TARGET: &str = "Error: invalid target (must be one of \"linux\", \"macos\", \"windows\")\n";

/// The `-ftarget` that is in effect: the last valid one before processing would stop (at `-h`,
/// `-v` or an invalid value). `-g` notes and the help page depend on it.
fn final_target(events: &[Event]) -> Target {
    let mut target = Target::host();
    for event in events {
        match event {
            Event::Help | Event::Version => break,
            Event::OptLevel(level) if parse_optimisation_level(level).is_err() => break,
            Event::CompileMode(mode) if parse_compile_mode(mode).is_none() => break,
            Event::Target(name) => match Target::from_name(name) {
                Some(t) => target = t,
                None => break,
            },
            _ => {}
        }
    }
    target
}

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

fn main() {
    let args: Vec<OsString> = std::env::args_os().collect();
    let program_name = args.first().map(os_to_bytes).unwrap_or_default();
    let program_name_str = String::from_utf8_lossy(&program_name).into_owned();

    let args = normalize(args);
    let cli = match Cli::try_parse_from(&args) {
        Ok(cli) => cli,
        Err(e) => {
            let _ = e.print();
            exit(1);
        }
    };
    let events = events(&args);
    let target = final_target(&events);

    let mut state = CompileState::new(target);
    if cli.debug > 0 {
        state.log_level = LogLevel::Debug;
    }
    state.martyrdom = cli.no_martyrdom == 0;

    for event in &events {
        match event {
            Event::Help => {
                print_help_page(&program_name_str, target);
                exit(0);
            }
            Event::Version => {
                //Print the version number, but without the "v" at the beginning
                out_str(&format!("{}\n", &frontend::log::VERSION_STRING[1..]));
                exit(0);
            }
            Event::OptLevel(level) => match parse_optimisation_level(level) {
                Ok(level) => state.optimisation_level = level,
                Err(msg) => {
                    err_str(&msg);
                    exit(1);
                }
            },
            Event::CompileMode(mode) => match parse_compile_mode(mode) {
                Some(mode) => state.compile_mode = mode,
                None => {
                    err_str("Error: invalid compile mode (must be one of \"noob\", \"bully\", \"obfuscated\")\n");
                    exit(1);
                }
            },
            Event::Target(name) => {
                if Target::from_name(name).is_none() {
                    err_str(INVALID_TARGET);
                    exit(1);
                }
            }
            Event::Assembly => state.output_mode = OutputMode::AssemblyFile,
            Event::Object => state.output_mode = OutputMode::ObjectFile,
            Event::DebugInfo => match target {
                //On Windows and MacOS, -g does not work - output a note, but don't do anything
                Target::Windows => print_note("-g cannot be used on Windows-systems, this option will be ignored.", false, &[]),
                Target::MacOS => print_note("-g cannot be used on MacOS-systems, this option will be ignored.", false, &[]),
                Target::Linux => state.use_debug_info = true,
            },
        }
    }

    if state.use_debug_info && state.compile_mode == CompileMode::Bully {
        print_note("-g cannot be used in bully mode, this option will be ignored.", false, &[]);
        state.use_debug_info = false;
    }

    let Some(output_file) = cli.output.last().map(os_to_bytes) else {
        err_str("Error: No output file specified\n");
        print_explanation_message(&program_name);
        exit(1);
    };
    if cli.files.is_empty() {
        err_str("Error: No input file(s) specified\n");
        print_explanation_message(&program_name);
        exit(1);
    }

    //Open each file one by one and parse it
    for input in &cli.files {
        let name = os_to_bytes(input);
        let mut file = match std::fs::File::open(bytes_to_path(&name)) {
            Ok(f) => f,
            Err(e) => {
                err_str(&format!("Failed to open input file: {}\n", frontend::libc::io_error_message(&e)));
                print_explanation_message(&program_name);
                exit(1);
            }
        };

        //Check that this is a regular file. If we did not check for this, an input file like "/dev/urandom" would pass without errors
        if !file.metadata().map(|m| m.is_file()).unwrap_or(false) {
            err(b"Error while opening input file: Your provided file name does not point to a regular file (e.g. it could be a directory, character device or a socket)\n");
            print_explanation_message(&program_name);
            exit(1);
        }

        print_debug(state.log_level, "Opening file \"%s\" successful, parsing file...", &[Arg::S(&name)]);
        let mut data = Vec::new();
        if let Err(e) = file.read_to_end(&mut data) {
            err_str(&format!("Failed to read input file: {}\n", frontend::libc::io_error_message(&e)));
            exit(1);
        }
        let data = parser::file_parser::host_text_mode_input(data);
        let parsed = parser::parse_file(&name, &data, &mut state);
        state.files.push(parsed);
        print_debug(state.log_level, "File parsing done, closing file...", &[]);
    }

    compile(state, &output_file);
}
