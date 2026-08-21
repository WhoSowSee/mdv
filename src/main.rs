use anyhow::Result;
use clap::{CommandFactory, FromArgMatches, ValueEnum};
use mdv::{
    cli::{Cli, LineNumberOptions},
    run,
};
use std::ffi::OsString;

fn main() -> Result<()> {
    env_logger::init();

    let matches = Cli::command().get_matches_from(normalize_line_number_args(std::env::args_os()));
    let cli = Cli::from_arg_matches(&matches)?;
    run(cli, &matches)
}

fn normalize_line_number_args(args: impl IntoIterator<Item = OsString>) -> Vec<OsString> {
    let mut args = args.into_iter().peekable();
    let mut normalized = Vec::new();

    while let Some(mut argument) = args.next() {
        let has_explicit_mode = args
            .peek()
            .and_then(|value| value.to_str())
            .is_some_and(|value| LineNumberOptions::from_str(value, false).is_ok());
        if !has_explicit_mode {
            if argument == "--line-numbers" || argument == "-N" {
                argument = "--line-numbers=rendered".into();
            } else if argument == "--code-line-numbers" || argument == "-K" {
                argument = "--code-line-numbers=rendered".into();
            }
        }
        normalized.push(argument);
    }
    normalized
}
