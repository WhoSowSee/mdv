use anyhow::Result;
use clap::{CommandFactory, FromArgMatches};
use mdv::{cli::Cli, run};
use std::io::IsTerminal;

fn main() -> Result<()> {
    env_logger::init();

    if std::env::args_os().len() == 1 && std::io::stdin().is_terminal() {
        Cli::command().print_long_help()?;
        println!();
        return Ok(());
    }

    let matches = Cli::command().get_matches();
    let cli = Cli::from_arg_matches(&matches)?;
    run(cli, &matches)
}
