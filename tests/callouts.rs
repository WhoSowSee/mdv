use assert_cmd::Command;
use std::fs;
use tempfile::NamedTempFile;

#[path = "callouts/basic.rs"]
mod basic;
#[path = "callouts/customization.rs"]
mod customization;
#[path = "callouts/formatting.rs"]
mod formatting;
#[path = "callouts/heading_layout.rs"]
mod heading_layout;
#[path = "callouts/tables_links.rs"]
mod tables_links;
fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}

fn render_callout_table(callout_style: &str, table_smart_indent: bool) -> String {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> [!note] Table in callout\n> Text\n>\n> | A | B |\n> | --- | --- |\n> | one | two |\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg(callout_style);
    if table_smart_indent {
        cmd.arg("--table-smart-indent");
    }

    let output = cmd
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout table");
    assert!(output.status.success());

    String::from_utf8(output.stdout).expect("stdout utf8")
}

fn is_empty_box_line(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with('│') || !trimmed.ends_with('│') {
        return false;
    }

    let inner: String = trimmed
        .chars()
        .skip(1)
        .take(trimmed.chars().count().saturating_sub(2))
        .collect();

    inner.trim().is_empty()
}

fn spaces_after_prefix(line: &str, prefix: char) -> usize {
    let mut chars = line.chars();
    let first = chars.next().expect("line not empty");
    assert_eq!(
        first, prefix,
        "expected prefix '{}' in line: {}",
        prefix, line
    );
    let mut count = 0usize;
    for ch in chars {
        if ch == ' ' {
            count += 1;
        } else {
            break;
        }
    }
    count
}
