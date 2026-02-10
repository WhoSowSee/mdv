use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

#[path = "links_tables/basic.rs"]
mod basic;
#[path = "links_tables/code_blocks.rs"]
mod code_blocks;
#[path = "links_tables/references.rs"]
mod references;
#[path = "links_tables/smart_indent.rs"]
mod smart_indent;
#[path = "links_tables/truncation.rs"]
mod truncation;

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}
