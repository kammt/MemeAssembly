//! Breakfiles, the MemeAssembly Makefiles. If `memeasm` is invoked without input files, it
//! compiles the files listed in the file `Breakfile` of the current directory.
//!
//! A Breakfile lists one source file per line, as `the secret ingredient is <path>`. The path
//! (relative to the current directory, or absolute) is everything after that prefix. Empty
//! lines and comments are not allowed, and the file must not end with a newline.

use super::commands::COMMENT_START;

/// The name of the Breakfile that is looked for in the current directory.
pub const BREAKFILE_NAME: &str = "Breakfile";

/// What every line of a Breakfile starts with.
const INGREDIENT: &[u8] = b"the secret ingredient is ";

/// A mistake in a Breakfile, at line `line` (counted from 1).
#[derive(Debug, PartialEq, Eq)]
pub struct BreakfileError {
    pub line: usize,
    pub message: &'static str,
}

/// The input file paths listed in the Breakfile `data`, in order, or every mistake in it.
/// Lines may end with `\n` or `\r\n`.
pub fn parse(data: &[u8]) -> Result<Vec<&[u8]>, Vec<BreakfileError>> {
    let error = |line, message| BreakfileError { line, message };
    if data.is_empty() {
        return Err(vec![error(1, "a Breakfile needs at least one secret ingredient")]);
    }

    let mut lines: Vec<&[u8]> = data
        .split(|&b| b == b'\n')
        .map(|line| line.strip_suffix(b"\r").unwrap_or(line))
        .collect();
    // The "line" after a final newline is not a line. A lone CR at the very end is a newline
    // as well.
    let ends_with_newline = matches!(data.last(), Some(b'\n' | b'\r'));
    if data.ends_with(b"\n") {
        lines.pop();
    }

    let mut errors = Vec::new();

    let mut files = Vec::new();
    for (index, &line) in lines.iter().enumerate() {
        let line_num = index + 1;
        if line.is_empty() {
            errors.push(error(line_num, "empty lines are not allowed in a Breakfile"));
        } else if line.starts_with(COMMENT_START) {
            errors.push(error(line_num, "comments are not allowed in a Breakfile"));
        } else if let Some(path) = line.strip_prefix(INGREDIENT).filter(|path| !path.is_empty()) {
            files.push(path);
        } else if line == INGREDIENT.trim_ascii_end() || line == INGREDIENT {
            errors.push(error(line_num, "the secret ingredient is missing"));
        } else {
            errors.push(error(line_num, "expected 'the secret ingredient is <file>'"));
        }
    }

    if ends_with_newline {
        errors.push(error(lines.len(), "a Breakfile must not end with a newline"));
    }

    if errors.is_empty() {
        Ok(files)
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn errors(data: &str) -> Vec<(usize, &'static str)> {
        parse(data.as_bytes()).unwrap_err().into_iter().map(|e| (e.line, e.message)).collect()
    }

    #[test]
    fn ingredients() {
        let data = b"the secret ingredient is main.memeasm\nthe secret ingredient is helper.memeasm\r\n\
                     the secret ingredient is /abs/path with spaces/x.memeasm";
        let files = parse(data).unwrap();
        assert_eq!(files, [&b"main.memeasm"[..], b"helper.memeasm", b"/abs/path with spaces/x.memeasm"]);
        assert_eq!(parse(b"the secret ingredient is a/subdir/f.memeasm").unwrap(), [b"a/subdir/f.memeasm"]);
    }

    #[test]
    fn trailing_newline() {
        let message = "a Breakfile must not end with a newline";
        assert_eq!(errors("the secret ingredient is a\n"), [(1, message)]);
        assert_eq!(errors("the secret ingredient is a\nthe secret ingredient is b\r\n"), [(2, message)]);
        assert_eq!(errors("the secret ingredient is a\r"), [(1, message)]);
        assert_eq!(errors("\n"), [(1, "empty lines are not allowed in a Breakfile"), (1, message)]);
    }

    #[test]
    fn mistakes() {
        assert_eq!(errors(""), [(1, "a Breakfile needs at least one secret ingredient")]);
        assert_eq!(
            errors(
                "the secret ingredient is a\n\nWhat the hell happened here? b\nthe secret ingredient is \n\
                 the secret ingredient is\nThe secret ingredient is c\nthe secret ingredient is d"
            ),
            [
                (2, "empty lines are not allowed in a Breakfile"),
                (3, "comments are not allowed in a Breakfile"),
                (4, "the secret ingredient is missing"),
                (5, "the secret ingredient is missing"),
                (6, "expected 'the secret ingredient is <file>'"),
            ]
        );
    }
}
