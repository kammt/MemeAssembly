//! Command line definition.
//!
//! clap parses the complete command line before anything is acted on. `-h` and `-v` are
//! handled by `main` (the help page is MemeAssembly's own), and every error exits with code 1.
//! A repeated option is not an error: its last occurrence wins.

use std::ffi::OsString;

use clap::{Parser, ValueEnum};

use super::commands::{CompileMode, OPT_1, OPT_2, OPT_3, OPT_69420, OPT_NONE, OPT_S};
use crate::target::Target;

#[derive(Parser, Debug)]
#[command(
    name = "memeasm",
    about = "The MemeAssembly compiler",
    disable_help_flag = true,
    disable_version_flag = true,
    args_override_self = true
)]
pub struct Cli {
    /// Output file
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    pub output: Option<OsString>,

    /// Only compile to x86_64 Assembly code
    #[arg(short = 'S', long = "assembly", conflicts_with = "object")]
    pub assembly: bool,

    /// Only compile to an object file
    #[arg(short = 'O', long = "object")]
    pub object: bool,

    /// Reverse optimisation stage -1, -2, -3, reverse storage optimisation -s, or the maximum
    /// optimisation 69420
    #[arg(long = "opt-level", value_name = "LEVEL", allow_hyphen_values = true)]
    pub opt_level: Option<OptLevel>,

    /// Compile mode
    #[arg(long = "compile-mode", value_name = "MODE", default_value = "noob")]
    pub compile_mode: CompileMode,

    /// Disables martyrdom
    #[arg(long = "no-martyrdom")]
    pub no_martyrdom: bool,

    /// Target operating system (default: the host's)
    #[arg(long = "target", value_name = "OS")]
    pub target: Option<Target>,

    /// Enables debug logs
    #[arg(short = 'd', long = "debug")]
    pub debug: bool,

    /// Write debug info into the compiled file
    #[arg(short = 'g', long = "debug-info")]
    pub debug_info: bool,

    /// Displays the help page
    #[arg(short = 'h', long = "help")]
    pub help: bool,

    /// Prints version information
    #[arg(short = 'v', long = "version")]
    pub version: bool,

    /// Input files
    #[arg(value_name = "inputFile")]
    pub files: Vec<OsString>,
}

/// The values of `--opt-level`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OptLevel {
    #[value(name = "-1")]
    Reverse1,
    #[value(name = "-2")]
    Reverse2,
    #[value(name = "-3")]
    Reverse3,
    #[value(name = "-s")]
    ReverseStorage,
    #[value(name = "69420")]
    Max,
}

/// The `optimisation_level` of an optional `--opt-level`.
pub fn optimisation_level(level: Option<OptLevel>) -> i64 {
    match level {
        None => OPT_NONE,
        Some(OptLevel::Reverse1) => OPT_1,
        Some(OptLevel::Reverse2) => OPT_2,
        Some(OptLevel::Reverse3) => OPT_3,
        Some(OptLevel::ReverseStorage) => OPT_S,
        Some(OptLevel::Max) => OPT_69420,
    }
}

/// The command line name of a target (`linux`, `macos`, `windows`).
pub fn target_name(target: Target) -> String {
    target.to_possible_value().map(|v| v.get_name().to_string()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(std::iter::once("memeasm").chain(args.iter().copied()))
    }

    #[test]
    fn options() {
        let cli = parse(&[
            "in1", "--opt-level", "-1", "-o", "out", "-S", "--compile-mode", "bully", "in2", "-g", "-d",
            "--target=windows", "--no-martyrdom",
        ])
        .unwrap();
        assert_eq!(cli.files, ["in1", "in2"]);
        assert_eq!(cli.output.as_deref(), Some("out".as_ref()));
        assert_eq!(optimisation_level(cli.opt_level), OPT_1);
        assert_eq!(cli.compile_mode, CompileMode::Bully);
        assert_eq!(cli.target, Some(Target::Windows));
        assert!(cli.assembly && !cli.object && cli.debug_info && cli.debug && cli.no_martyrdom);
        assert!(!cli.help && !cli.version);
    }

    #[test]
    fn defaults() {
        let cli = parse(&["-o", "out", "in"]).unwrap();
        assert_eq!(optimisation_level(cli.opt_level), OPT_NONE);
        assert_eq!(cli.compile_mode, CompileMode::Noob);
        assert_eq!(cli.target, None);
        assert!(!cli.assembly && !cli.object && !cli.no_martyrdom);
    }

    #[test]
    fn opt_levels() {
        for (value, level) in [("-1", OPT_1), ("-2", OPT_2), ("-3", OPT_3), ("-s", OPT_S), ("69420", OPT_69420)] {
            assert_eq!(optimisation_level(parse(&["--opt-level", value]).unwrap().opt_level), level);
            let attached = format!("--opt-level={value}");
            assert_eq!(optimisation_level(parse(&[&attached]).unwrap().opt_level), level);
        }
        assert_eq!(parse(&["--opt-level=-4"]).unwrap_err().kind(), ErrorKind::InvalidValue);
    }

    #[test]
    fn short_clusters_and_attached_values() {
        let cli = parse(&["-dSoout", "in"]).unwrap();
        assert!(cli.debug && cli.assembly);
        assert_eq!(cli.output.as_deref(), Some("out".as_ref()));
        let cli = parse(&["-gO", "-o", "out.o", "in"]).unwrap();
        assert!(cli.debug_info && cli.object);
        assert_eq!(cli.output.as_deref(), Some("out.o".as_ref()));
    }

    #[test]
    fn last_occurrence_wins() {
        let cli = parse(&["--opt-level=-1", "--opt-level=69420", "-o", "a", "-o", "b", "-g", "-g"]).unwrap();
        assert_eq!(optimisation_level(cli.opt_level), OPT_69420);
        assert_eq!(cli.output.as_deref(), Some("b".as_ref()));
    }

    #[test]
    fn errors() {
        let kind = |args: &[&str]| parse(args).unwrap_err().kind();
        assert_eq!(kind(&["-S", "-O"]), ErrorKind::ArgumentConflict);
        assert_eq!(kind(&["-x"]), ErrorKind::UnknownArgument);
        assert_eq!(kind(&["-fcompile-mode", "bully"]), ErrorKind::UnknownArgument);
        assert_eq!(kind(&["--compile-mode", "foo"]), ErrorKind::InvalidValue);
        assert_eq!(kind(&["--target", "amiga"]), ErrorKind::InvalidValue);
        assert_eq!(kind(&["in", "-o"]), ErrorKind::InvalidValue);
        assert_eq!(kind(&["in", "--compile-mode"]), ErrorKind::InvalidValue);
    }

    #[test]
    fn terminator() {
        let cli = parse(&["-o", "out", "--", "-S"]).unwrap();
        assert_eq!(cli.files, ["-S"]);
        assert!(!cli.assembly);
    }

    #[test]
    fn target_names() {
        assert_eq!(target_name(Target::MacOS), "macos");
        assert_eq!(target_name(Target::Linux), "linux");
        assert_eq!(target_name(Target::Windows), "windows");
    }
}
