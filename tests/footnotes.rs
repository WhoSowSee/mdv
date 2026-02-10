use assert_cmd::Command;
use mdv::utils::strip_ansi;
use std::fs;
use tempfile::NamedTempFile;

#[path = "footnotes/attached.rs"]
mod attached;
#[path = "footnotes/ordering.rs"]
mod ordering;
#[path = "footnotes/placement.rs"]
mod placement;
#[path = "footnotes/validation.rs"]
mod validation;

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}
