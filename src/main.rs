use anyhow::Result;
use clap::{CommandFactory, FromArgMatches};
use mdv::{cli::Cli, run};

fn main() -> Result<()> {
    env_logger::init();

    let matches = Cli::command().get_matches();
    let cli = Cli::from_arg_matches(&matches)?;
    run(cli, &matches)
}
