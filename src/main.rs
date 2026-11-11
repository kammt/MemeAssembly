//! The MemeAssembly compiler (port of compiler/memeasm.c).

mod backend;
mod frontend;
mod target;

use std::ffi::OsString;
use std::io::Read;
use std::process::exit;

use clap::Parser;

use frontend::cli::{optimisation_level, target_name, Cli};
use frontend::commands::{CompileMode, CompileState, LogLevel, OutputMode};
use frontend::log::{err, err_str, out_str, print_debug, print_information_header, print_note, Arg};
use frontend::{bytes_to_path, compile, parser, print_explanation_message};
use target::Target;

/// Prints the help page of this command. Launched by using the -h option in the terminal
fn print_help_page(program_name: &str, target: Target) {
    print_information_header(target);
    let p = program_name;
    out_str("Usage:\n");
    out_str(&format!(" {p} [options] -o outputFile [-d] inputFile...\t\tCompiles the specified file(s) into an executable\n"));
    out_str(&format!(" {p} [options] -S -o outputFile.S [-d] inputFile...\tOnly compiles the specified file(s) and saves them as x86_64 Assembly code\n"));
    out_str(&format!(" {p} [options] -O -o outputFile.o [-d] inputFile...\tOnly compiles the specified file(s) and saves them as an object file\n"));
    out_str(&format!(" {p} (-h | --help)\t\t\t\t\t\tDisplays this help page\n"));
    out_str(&format!(" {p} (-v | --version)\t\t\t\t\tPrints version information\n\n"));
    out_str("Compiler options:\n");
    out_str(" --opt-level=-1 \t- reverse optimisation stage 1: A nop is inserted after every command\n");
    out_str(" --opt-level=-2 \t- reverse optimisation stage 2: A register is moved to and from the Stack after every command\n");
    out_str(" --opt-level=-3 \t- reverse optimisation stage 3: A xmm-register is moved to and from the Stack using movups after every command\n");
    out_str(" --opt-level=-s \t- reverse storage optimisation: Intentionally increases the file size by aligning end of the compiled Assembly-code to 536870912B\n");
    out_str(" --opt-level=69420 \t- maximum optimisation. Reduces the execution to close to 0s by optimising out your entire code\n");
    out_str(" --compile-mode=<noob|bully|obfuscated> - Change the compile mode (default: noob)\n");
    out_str(" -g \t\t\t- write debug info into the compiled file. Currently, only the DWARF format is supported (Linux-only)\n");
    out_str(" --no-martyrdom \t- Disables martyrdom\n");
    out_str(" -d \t\t\t- enables debug logs\n");
    out_str(&format!(
        " --target=<linux|macos|windows> - Change the target operating system (default: {})\n",
        target_name(Target::host())
    ));
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
            Target::Windows => print_note("-g cannot be used on Windows-systems, this option will be ignored.", false, &[]),
            Target::MacOS => print_note("-g cannot be used on MacOS-systems, this option will be ignored.", false, &[]),
            Target::Linux => state.use_debug_info = true,
        }
    }

    if state.use_debug_info && state.compile_mode == CompileMode::Bully {
        print_note("-g cannot be used in bully mode, this option will be ignored.", false, &[]);
        state.use_debug_info = false;
    }

    let Some(output_file) = cli.output.as_ref().map(os_to_bytes) else {
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
        let parsed = parser::parse_file(&name, &data, &mut state);
        state.files.push(parsed);
        print_debug(state.log_level, "File parsing done, closing file...", &[]);
    }

    compile(state, &output_file);
}
