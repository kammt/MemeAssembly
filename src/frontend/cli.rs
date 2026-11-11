//! Command line definition.
//!
//! The C compiler used `getopt_long_only("o:hO::dgSv", ...)`, which accepts spellings that
//! clap does not: long options with a single dash (`-fcompile-mode bully`, `-help`),
//! unambiguous abbreviations (`-fc`, `--out`), short clusters with an attached value
//! (`-So out`, `-ofile`) and `-O` with an *optional, attached* value (`-O` vs `-O-1`).
//! [`normalize`] rewrites exactly these forms into canonical clap spellings before parsing:
//!
//! * `-name[=v]` / `--name[=v]` where `name` is one of the C long options or an unambiguous
//!   prefix of one → `--canonical[=v]`; a required value given as the next argument is
//!   attached (`--output=<next>`), so it may start with `-` like with getopt.
//! * any other `-xyz` → its short options one by one: `-O<rest>` → `--opt-level=<rest>`,
//!   a bare `-O` → `--object`, `-o<rest>` / `-o <next>` → `--output=...`, others stay `-x`.
//! * everything after `--` and all non-options are passed through unchanged.
//!
//! Besides the C spellings, the clap-style aliases `--compile-mode`, `--no-martyrdom`,
//! `--target`, `--object`, `--opt-level` and `--version` are accepted.

use std::ffi::OsString;

use clap::{ArgAction, Parser};

#[derive(Parser, Debug)]
#[command(
    name = "memeasm",
    about = "The MemeAssembly compiler",
    disable_help_flag = true,
    disable_version_flag = true
)]
pub struct Cli {
    /// Output file
    #[arg(short = 'o', long = "output", value_name = "FILE", action = ArgAction::Append)]
    pub output: Vec<OsString>,

    /// Only compile to x86_64 Assembly code
    #[arg(short = 'S', action = ArgAction::Count)]
    pub assembly: u8,

    /// Only compile to an object file (`-O` without a value)
    #[arg(long = "object", action = ArgAction::Count)]
    pub object: u8,

    /// Reverse optimisation level (`-O-1`, `-O-2`, `-O-3`, `-O-s`, `-O69420`)
    #[arg(long = "opt-level", value_name = "LEVEL", allow_hyphen_values = true, action = ArgAction::Append)]
    pub opt_level: Vec<String>,

    /// Enables debug logs
    #[arg(short = 'd', long = "debug", action = ArgAction::Count)]
    pub debug: u8,

    /// Write debug info into the compiled file
    #[arg(short = 'g', action = ArgAction::Count)]
    pub debug_info: u8,

    /// Displays the help page
    #[arg(short = 'h', long = "help", action = ArgAction::Count)]
    pub help: u8,

    /// Prints version information
    #[arg(short = 'v', long = "version", action = ArgAction::Count)]
    pub version: u8,

    /// Compile mode: noob (default), bully or obfuscated
    #[arg(long = "fcompile-mode", alias = "compile-mode", value_name = "MODE", allow_hyphen_values = true, action = ArgAction::Append)]
    pub compile_mode: Vec<String>,

    /// Disables martyrdom
    #[arg(long = "fno-martyrdom", alias = "no-martyrdom", action = ArgAction::Count)]
    pub no_martyrdom: u8,

    /// Target operating system: linux, macos or windows
    #[arg(long = "ftarget", alias = "target", value_name = "OS", allow_hyphen_values = true, action = ArgAction::Append)]
    pub target: Vec<String>,

    /// Input files
    #[arg(value_name = "inputFile")]
    pub files: Vec<OsString>,
}

/// The long options of memeasm.c (plus `ftarget`), in getopt's table order, and whether they
/// take a value. Abbreviations are resolved against these names only.
const LONG_OPTIONS: [(&str, bool); 6] = [
    ("output", true),
    ("help", false),
    ("debug", false),
    ("fno-martyrdom", false),
    ("fcompile-mode", true),
    ("ftarget", true),
];

/// Additional exact-only long names (clap-style aliases), and whether they take a value.
const ALIASES: [(&str, bool); 6] = [
    ("compile-mode", true),
    ("no-martyrdom", false),
    ("target", true),
    ("object", false),
    ("opt-level", true),
    ("version", false),
];

/// The short options of the C optstring `o:hO::dgSv`.
const SHORT_OPTIONS: &[u8] = b"ohOdgSv";

enum LongMatch {
    Found(&'static str, bool),
    Ambiguous,
    None,
}

fn find_long(name: &str) -> LongMatch {
    if let Some(&(n, v)) = LONG_OPTIONS.iter().chain(ALIASES.iter()).find(|(n, _)| *n == name) {
        return LongMatch::Found(n, v);
    }
    let mut candidates = LONG_OPTIONS.iter().filter(|(n, _)| n.starts_with(name));
    match (candidates.next(), candidates.next()) {
        (Some(&(n, v)), None) if !name.is_empty() => LongMatch::Found(n, v),
        (Some(_), Some(_)) => LongMatch::Ambiguous,
        _ => LongMatch::None,
    }
}

/// Rewrites getopt_long_only spellings into canonical clap spellings (see the module docs).
pub fn normalize(args: Vec<OsString>) -> Vec<OsString> {
    let mut out = Vec::with_capacity(args.len());
    let mut iter = args.into_iter();
    if let Some(program) = iter.next() {
        out.push(program);
    }
    while let Some(arg) = iter.next() {
        let Some(text) = arg.to_str().map(str::to_string) else {
            out.push(arg);
            continue;
        };
        if text == "--" {
            out.push(arg);
            out.extend(iter.by_ref());
            break;
        }
        if !text.starts_with('-') || text == "-" {
            out.push(arg);
            continue;
        }

        let double_dash = text.starts_with("--");
        let body = if double_dash { &text[2..] } else { &text[1..] };
        // getopt_long_only tries a single-dash argument as a long option unless it is a
        // single valid short option character.
        let try_long = double_dash || body.len() > 1 || !SHORT_OPTIONS.contains(&body.as_bytes()[0]);
        if try_long {
            let (name, value) = match body.split_once('=') {
                Some((n, v)) => (n, Some(v)),
                None => (body, None),
            };
            match find_long(name) {
                LongMatch::Found(canonical, takes_value) => {
                    let mut canon = format!("--{canonical}");
                    match value {
                        Some(v) => {
                            canon.push('=');
                            canon.push_str(v);
                            out.push(canon.into());
                        }
                        None if takes_value => match iter.next() {
                            Some(next) => {
                                let mut joined = OsString::from(canon + "=");
                                joined.push(next);
                                out.push(joined);
                            }
                            None => out.push(canon.into()),
                        },
                        None => out.push(canon.into()),
                    }
                    continue;
                }
                // Unknown or ambiguous: let clap report it.
                LongMatch::Ambiguous => {
                    out.push(arg);
                    continue;
                }
                LongMatch::None if double_dash || !SHORT_OPTIONS.contains(&body.as_bytes()[0]) => {
                    out.push(arg);
                    continue;
                }
                LongMatch::None => {}
            }
        }

        // A cluster of short options.
        for (i, c) in body.char_indices() {
            let rest = &body[i + c.len_utf8()..];
            match c {
                'O' if rest.is_empty() => out.push("--object".into()),
                'O' => out.push(format!("--opt-level={rest}").into()),
                'o' if !rest.is_empty() => out.push(format!("--output={rest}").into()),
                'o' => match iter.next() {
                    Some(next) => {
                        let mut joined = OsString::from("--output=");
                        joined.push(next);
                        out.push(joined);
                    }
                    None => out.push("-o".into()),
                },
                _ => {
                    out.push(format!("-{c}").into());
                    continue;
                }
            }
            break;
        }
    }
    out
}

/// The options whose relative order matters (memeasm.c handled them one by one in argv order).
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    Help,
    Version,
    OptLevel(String),
    CompileMode(String),
    Target(String),
    DebugInfo,
    Assembly,
    Object,
}

/// The order-dependent options of a [`normalize`]d command line that clap accepted, in order.
/// After normalisation every option is a separate `-x` or `--name[=value]` argument.
pub fn events(normalized: &[OsString]) -> Vec<Event> {
    let mut list = Vec::new();
    for arg in normalized.iter().skip(1) {
        let Some(arg) = arg.to_str() else { continue };
        if arg == "--" {
            break;
        }
        let (name, value) = match arg.split_once('=') {
            Some((n, v)) => (n, v.to_string()),
            None => (arg, String::new()),
        };
        list.push(match name {
            "-h" | "--help" => Event::Help,
            "-v" | "--version" => Event::Version,
            "-g" => Event::DebugInfo,
            "-S" => Event::Assembly,
            "--object" => Event::Object,
            "--opt-level" => Event::OptLevel(value),
            "--fcompile-mode" | "--compile-mode" => Event::CompileMode(value),
            "--ftarget" | "--target" => Event::Target(value),
            _ => continue,
        });
    }
    list
}

#[cfg(test)]
mod tests {
    use super::*;

    fn norm(args: &[&str]) -> Vec<String> {
        let mut v: Vec<OsString> = vec!["memeasm".into()];
        v.extend(args.iter().map(OsString::from));
        normalize(v).into_iter().skip(1).map(|s| s.into_string().unwrap()).collect()
    }

    #[test]
    fn c_spellings() {
        assert_eq!(norm(&["-o", "out", "in.memeasm"]), ["--output=out", "in.memeasm"]);
        assert_eq!(norm(&["-ofile"]), ["--output=file"]);
        assert_eq!(norm(&["-So", "-weird"]), ["-S", "--output=-weird"]);
        assert_eq!(norm(&["-O"]), ["--object"]);
        assert_eq!(norm(&["-O-1", "-O-s", "-O69420"]), ["--opt-level=-1", "--opt-level=-s", "--opt-level=69420"]);
        assert_eq!(norm(&["-dO-2"]), ["-d", "--opt-level=-2"]);
        assert_eq!(norm(&["-dgS"]), ["-d", "-g", "-S"]);
        assert_eq!(norm(&["-fcompile-mode", "bully"]), ["--fcompile-mode=bully"]);
        assert_eq!(norm(&["-fcompile-mode=bully"]), ["--fcompile-mode=bully"]);
        assert_eq!(norm(&["-fno-martyrdom"]), ["--fno-martyrdom"]);
        assert_eq!(norm(&["-ftarget", "windows"]), ["--ftarget=windows"]);
        assert_eq!(norm(&["-help", "--help", "-h"]), ["--help", "--help", "-h"]);
        assert_eq!(norm(&["-debug", "-v"]), ["--debug", "-v"]);
    }

    #[test]
    fn abbreviations() {
        assert_eq!(norm(&["-out", "x"]), ["--output=x"]);
        assert_eq!(norm(&["--out=x"]), ["--output=x"]);
        assert_eq!(norm(&["-fc", "noob"]), ["--fcompile-mode=noob"]);
        assert_eq!(norm(&["-fno"]), ["--fno-martyrdom"]);
        assert_eq!(norm(&["-he"]), ["--help"]);
        // "-f" is ambiguous, "-x" unknown: left for clap to reject
        assert_eq!(norm(&["-f"]), ["-f"]);
        assert_eq!(norm(&["-xyz"]), ["-xyz"]);
    }

    #[test]
    fn positionals_and_terminator() {
        assert_eq!(norm(&["a", "-", "-S", "--", "-O", "b"]), ["a", "-", "-S", "--", "-O", "b"]);
    }

    #[test]
    fn clap_parses_normalized_arguments() {
        let mut v: Vec<OsString> = vec!["memeasm".into()];
        v.extend(["in1", "-O-1", "-o", "out", "-S", "-fcompile-mode", "bully", "in2", "-g"].map(OsString::from));
        let cli = Cli::try_parse_from(normalize(v)).unwrap();
        assert_eq!(cli.files, ["in1", "in2"]);
        assert_eq!(cli.output, ["out"]);
        assert_eq!(cli.opt_level, ["-1"]);
        assert_eq!(cli.compile_mode, ["bully"]);
        assert_eq!((cli.assembly, cli.debug_info, cli.object), (1, 1, 0));
    }

    #[test]
    fn events_in_command_line_order() {
        let mut v: Vec<OsString> = vec!["memeasm".into()];
        v.extend(["-g", "-o", "-g", "-O", "x", "-gS", "-ftarget", "windows", "-O-1", "--", "-h"].map(OsString::from));
        assert_eq!(
            events(&normalize(v)),
            [
                Event::DebugInfo,
                Event::Object,
                Event::DebugInfo,
                Event::Assembly,
                Event::Target("windows".into()),
                Event::OptLevel("-1".into()),
            ]
        );
    }
}
