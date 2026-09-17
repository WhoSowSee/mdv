use crate::support::mdv_cmd;
use std::fs;
use tempfile::NamedTempFile;

/// Build the markdown checkbox demo used across tests.
fn checkbox_markdown() -> String {
    [
        "- [ ] unchecked",
        "- [x] done",
        "- [-] canceled",
        "- [?] question",
        "- [!] important",
        "- [/] in progress",
        "- [|] alt progress",
        "- [\\] backslash state",
    ]
    .join("\n")
        + "\n"
}

fn nested_list_markdown() -> &'static str {
    "- level one\n  - level two\n    - level three\n      - level four\n        - level five\n"
}

fn run(args: &[&str], markdown: &str) -> String {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, markdown).unwrap();
    let mut cmd = mdv_cmd();
    cmd.args(["--color", "never"]);
    for arg in args {
        cmd.arg(arg);
    }
    cmd.arg(temp_file.path());
    let output = cmd.output().expect("mdv executed");
    assert!(output.status.success(), "mdv failed: {:?}", output.status);
    String::from_utf8(output.stdout).expect("stdout utf8")
}

fn run_with_colors(args: &[&str], markdown: &str) -> String {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, markdown).unwrap();
    let mut cmd = mdv_cmd();
    cmd.args(["--color", "always"]);
    for arg in args {
        cmd.arg(arg);
    }
    cmd.arg(temp_file.path());
    let output = cmd.output().expect("mdv executed");
    assert!(output.status.success(), "mdv failed: {:?}", output.status);
    String::from_utf8(output.stdout).expect("stdout utf8")
}

#[path = "checkboxes/basic.rs"]
mod basic;
#[path = "checkboxes/colors.rs"]
mod colors;
#[path = "checkboxes/custom_states.rs"]
mod custom_states;
#[path = "checkboxes/layout_and_lists.rs"]
mod layout_and_lists;
