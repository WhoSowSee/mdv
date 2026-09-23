use crate::support::mdv_cmd;
use std::fs;
use std::time::Duration;
use tempfile::NamedTempFile;

#[path = "callouts/basic.rs"]
mod basic;
#[path = "callouts/boundaries.rs"]
mod boundaries;
#[path = "callouts/customization.rs"]
mod customization;
#[path = "callouts/dialects.rs"]
mod dialects;
#[path = "callouts/formatting.rs"]
mod formatting;
#[path = "callouts/heading_layout.rs"]
mod heading_layout;
#[path = "callouts/regressions.rs"]
mod regressions;
#[path = "callouts/tables_links.rs"]
mod tables_links;

fn render(source: &str, style: &str, extra: &[&str]) -> String {
    let file = NamedTempFile::new().unwrap();
    fs::write(&file, source).unwrap();
    let output = mdv_cmd()
        .args([
            "--color",
            "never",
            "--callout-style",
            style,
            "--no-code-guessing",
        ])
        .args(extra)
        .arg(file.path())
        .timeout(Duration::from_secs(5))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "style {style}, input of {} bytes starting with {:?}, status {}: {}",
        source.len(),
        source.lines().next(),
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn render_callout_table(callout_style: &str, table_smart_indent: bool) -> String {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> [!note] Table in callout\n> Text\n>\n> | A | B |\n> | --- | --- |\n> | one | two |\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.args(["--color", "never"])
        .arg("-w")
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
