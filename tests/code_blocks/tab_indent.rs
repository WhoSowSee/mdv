use assert_cmd::Command;

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}
use std::fs;
use tempfile::NamedTempFile;

#[path = "tab_indent/deep_indent.rs"]
mod deep_indent;
#[path = "tab_indent/fences.rs"]
mod fences;
#[path = "tab_indent/paragraphs.rs"]
mod paragraphs;
