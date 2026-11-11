//! Port of compiler/{compiler.c,parser,analyser,translator,logger}.
//!
//! The pipeline is the same as in C: [`parser::parse_file`] for every input file, then
//! [`compile`], which runs the analysis and then either writes the assembly text of
//! [`translator`] (`-S`) or generates the code directly with [`encoder`] and hands it to
//! [`crate::backend::emit`] (replacing the original `popen("gcc ...")`).

pub mod analyser;
pub mod cli;
pub mod commands;
pub mod encoder;
pub mod libc;
pub mod log;
pub mod parser;
pub mod translator;

use std::path::PathBuf;

use crate::backend::{self, OutputKind};
use commands::{CompileState, OutputMode};
use log::{err, err_str, exit, print_error_ascii};

/// Converts a C-style byte string (file name from argv) into a path.
pub fn bytes_to_path(b: &[u8]) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        PathBuf::from(std::ffi::OsStr::from_bytes(b))
    }
    #[cfg(not(unix))]
    {
        PathBuf::from(String::from_utf8_lossy(b).into_owned())
    }
}

/// Analyses and translates the parsed files and writes the requested output. Never returns.
pub fn compile(mut state: CompileState, output_file_name: &[u8]) -> ! {
    analyser::analyse_commands(&mut state);

    if state.compiler_errors > 0 {
        print_error_ascii();
        err_str(&format!(
            "Compilation failed with {} error(s), please check your code and try again.\n",
            state.compiler_errors
        ));
        exit(1);
    }

    let output_path = bytes_to_path(output_file_name);
    if state.output_mode == OutputMode::AssemblyFile {
        // The C code opens the output file before translating.
        let mut file = match std::fs::File::create(&output_path) {
            Ok(f) => f,
            Err(e) => {
                err_str(&format!("Failed to open output file: {}\n", libc::io_error_message(&e)));
                exit(1);
            }
        };
        let asm = translator::write_to_string(&state);
        use std::io::Write;
        if let Err(e) = file.write_all(&log::text_mode(&asm)) {
            err_str(&format!("Failed to write output file: {}\n", libc::io_error_message(&e)));
            exit(1);
        }
        exit(0);
    }

    let kind = if state.output_mode == OutputMode::ObjectFile {
        OutputKind::Object
    } else {
        OutputKind::Executable
    };
    // The C code piped the assembly text into gcc; here the program is generated directly.
    let module = match encoder::encode_module(&state) {
        Ok(module) => module,
        Err(msg) => {
            let msg = msg.strip_prefix("Error: ").unwrap_or(&msg);
            err_str(&format!("{{standard input}}: Assembler messages:\n{{standard input}}: Error: {msg}\n"));
            assembling_failed();
        }
    };
    if let Err(msg) = backend::emit(&module, state.target, kind, &output_path) {
        let mut m = msg.into_bytes();
        if m.last() != Some(&b'\n') {
            m.push(b'\n');
        }
        err(&m);
        assembling_failed();
    }
    exit(0);
}

fn assembling_failed() -> ! {
    err_str("Assembling/linking failed. If you did not expect this to happen, please report this issue at https://github.com/kammt/MemeAssembly/issues so that it can be fixed\n");
    exit(1);
}

/// `printf` of the explanation line shown after CLI errors.
pub fn print_explanation_message(program_name: &[u8]) {
    let mut m = b"Usage: ".to_vec();
    m.extend_from_slice(program_name);
    m.extend_from_slice(b" -o outputFile inputFile\n");
    log::out(&m);
}
