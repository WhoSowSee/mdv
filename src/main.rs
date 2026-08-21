use anyhow::Result;
use clap::{CommandFactory, FromArgMatches, ValueEnum};
use mdv::{
    cli::{Cli, LineNumberOptions},
    run,
};
use std::ffi::{OsStr, OsString};

fn main() -> Result<()> {
    env_logger::init();

    let matches =
        Cli::command().get_matches_from(normalize_code_line_number_args(std::env::args_os()));
    let cli = Cli::from_arg_matches(&matches)?;
    run(cli, &matches)
}

fn normalize_code_line_number_args(args: impl IntoIterator<Item = OsString>) -> Vec<OsString> {
    let mut args = args.into_iter().peekable();
    let mut normalized = Vec::new();
    let mut deferred_flags = Vec::new();

    while let Some(argument) = args.next() {
        let is_bare_flag = argument == "--code-line-numbers" || argument == "-K";
        if is_bare_flag
            && args
                .peek()
                .is_some_and(|next| code_line_number_token_is_file(next))
        {
            deferred_flags.push(argument);
        } else {
            normalized.push(argument);
        }
    }

    let insert_at = normalized
        .iter()
        .position(|argument| argument == "--")
        .unwrap_or(normalized.len());
    normalized.splice(insert_at..insert_at, deferred_flags);
    normalized
}

fn code_line_number_token_is_file(argument: &OsStr) -> bool {
    let Some(argument) = argument.to_str() else {
        return true;
    };
    if argument == "-" {
        return true;
    }
    if argument == "--" || argument.starts_with('-') {
        return false;
    }
    LineNumberOptions::from_str(argument, false).is_err()
}
