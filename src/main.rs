use anyhow::Result;
use clap::{CommandFactory, FromArgMatches, ValueEnum};
use mdv::{
    cli::{Cli, LineNumberOptions},
    run,
};
use std::ffi::OsString;

mod version;

fn main() -> Result<()> {
    match run_cli() {
        Err(error)
            if error
                .downcast_ref::<std::io::Error>()
                .is_some_and(|error| error.kind() == std::io::ErrorKind::BrokenPipe) =>
        {
            Ok(())
        }
        result => result,
    }
}

fn run_cli() -> Result<()> {
    env_logger::Builder::from_default_env()
        .write_style(env_logger::WriteStyle::Never)
        .init();

    let matches = match Cli::command()
        .try_get_matches_from(normalize_line_number_args(std::env::args_os()))
    {
        Ok(matches) => matches,
        Err(error) if error.kind() == clap::error::ErrorKind::DisplayVersion => {
            version::print()?;
            return Ok(());
        }
        Err(error) if error.kind() == clap::error::ErrorKind::DisplayHelp => {
            error.print()?;
            return Ok(());
        }
        Err(error) => error.exit(),
    };
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
