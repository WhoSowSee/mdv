use assert_cmd::Command;
use mdv::utils::{display_width, strip_ansi};
use predicates::prelude::*;
use std::fs;
use std::time::Duration;
use tempfile::{NamedTempFile, TempDir};

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}

fn mdv_cmd_with_config(config_dir: &TempDir) -> Command {
    let mut cmd = mdv_cmd();
    cmd.arg("--config-file").arg(config_dir.path());
    cmd
}

#[path = "cli_basic/config_and_pager.rs"]
mod config_and_pager;
#[path = "cli_basic/general.rs"]
mod general;
#[path = "cli_basic/html_content.rs"]
mod html_content;
#[path = "cli_basic/html_lists_tables.rs"]
mod html_lists_tables;
#[path = "cli_basic/html_semantics.rs"]
mod html_semantics;
#[path = "cli_basic/rendering_options.rs"]
mod rendering_options;
