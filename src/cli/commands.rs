use super::*;

#[derive(Debug, Clone, Copy, Subcommand)]
pub enum CliCommand {
    /// Show the full help in the selected pager
    Help,
}
