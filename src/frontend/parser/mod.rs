//! Port of compiler/parser/parser.c

pub mod file_parser;
pub mod function_parser;

use super::commands::{CompileState, File};

/// Parses the contents of one input file into a [`File`].
pub fn parse_file(file_name: &[u8], data: &[u8], state: &mut CompileState) -> File {
    let parsed_commands = file_parser::parse_commands(data, file_name, state);
    let mut file = File {
        file_name: file_name.to_vec(),
        loc: parsed_commands.len(),
        functions: Vec::new(),
        parsed_commands,
        random_index: 0,
    };
    function_parser::parse_functions(&mut file, state);
    file
}
