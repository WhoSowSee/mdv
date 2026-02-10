use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

#[path = "layout/blockquotes.rs"]
mod blockquotes;
#[path = "layout/headings.rs"]
mod headings;
#[path = "layout/spacing.rs"]
mod spacing;

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}
