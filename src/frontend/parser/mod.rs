//! Port of compiler/parser/parser.c

pub mod file_parser;
pub mod function_parser;

use super::state::{CompileState, File};

/// Parses the contents of one input file into a [`File`].
pub fn parse_file(file_name: &[u8], data: &[u8], state: &mut CompileState) -> File {
    let commands = file_parser::parse_commands(data, file_name, state);
    let loc = commands.len();
    let (functions, orphans) = function_parser::parse_functions(file_name, commands, state);
    File { file_name: file_name.to_vec(), loc, functions, orphans, random_index: 0 }
}
