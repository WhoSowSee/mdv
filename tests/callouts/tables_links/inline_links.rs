use super::*;

#[test]
fn test_callout_pretty_reference_marker_is_not_split_from_url() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> [!info] Dense links in callout\n> | Topic | Link Set |\n> | --- | --- |\n> | Terminal | [osc8](https://iterm2.com/feature-reporting/Hyperlinks_in_Terminal_Emulators.html) |\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("--cols")
        .arg("60")
        .arg("--wrap")
        .arg("word")
        .arg("--link-style")
        .arg("inlinetable")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for pretty callout long reference");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(
        lines
            .iter()
            .any(|line| line.contains("[1] https://iterm2.com")),
        "expected reference marker and URL start on the same line, stdout:\n{}",
        stdout
    );

    let marker_only_idx = lines.iter().position(|line| {
        let trimmed = line.trim();
        trimmed == "│ [1] │" || trimmed == "│ [1]  │" || trimmed == "[1]"
    });
    if let Some(idx) = marker_only_idx {
        let next_trimmed = lines.get(idx + 1).copied().unwrap_or_default().trim();
        assert!(
            !next_trimmed.starts_with("https://"),
            "reference marker must not be split into its own line, stdout:\n{}",
            stdout
        );
    }
}

#[test]
fn test_callout_inline_links_render_outside_while_table_links_stay_inside() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> [!info] Dense links in callout\n> Testing smart indent together with callout gutters and inline [references](example.com).\n>\n> | Topic | Link Set |\n> | --- | --- |\n> | Terminal | [osc8](https://iterm2.com/feature-reporting/Hyperlinks_in_Terminal_Emulators.html) |\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("--cols")
        .arg("90")
        .arg("--wrap")
        .arg("word")
        .arg("--link-style")
        .arg("inlinetable")
        .arg("--callout-style")
        .arg("pretty")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for mixed callout links");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(
        stdout.contains("references[1]"),
        "expected callout inline link marker in body text, stdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("osc8[1]"),
        "expected table link marker to use table-local numbering, stdout:\n{}",
        stdout
    );
    assert!(
        lines.contains(&"[1] example.com"),
        "expected callout-level reference block outside callout, stdout:\n{}",
        stdout
    );
    assert!(
        !lines.iter().any(|line| line.contains("│ [1] example.com")),
        "callout-level reference must not be rendered inside pretty callout, stdout:\n{}",
        stdout
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("│ [1] https://iterm2.com/feature-reporting/")),
        "expected table reference line to stay inside callout box, stdout:\n{}",
        stdout
    );
}
