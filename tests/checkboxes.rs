use assert_cmd::Command;
use std::fs;
use tempfile::NamedTempFile;

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}

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

fn run(args: &[&str], markdown: &str) -> String {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, markdown).unwrap();
    let mut cmd = mdv_cmd();
    cmd.arg("--no-colors");
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
    for arg in args {
        cmd.arg(arg);
    }
    cmd.arg(temp_file.path());
    let output = cmd.output().expect("mdv executed");
    assert!(output.status.success(), "mdv failed: {:?}", output.status);
    String::from_utf8(output.stdout).expect("stdout utf8")
}

#[test]
fn test_pretty_checkbox_square_icons() {
    let stdout = run(&["--pretty-checkbox", "square"], &checkbox_markdown());
    let icons = [
        ('\u{F0131}', "unchecked"),
        ('\u{F0132}', "done"),
        ('\u{F0375}', "canceled"),
        ('\u{F078B}', "question"),
        ('\u{F0027}', "important"),
        ('\u{F0856}', "in progress"),
        ('\u{F0856}', "alt progress"),
        ('\u{F0856}', "backslash state"),
    ];
    for (icon, label) in icons {
        let line = stdout
            .lines()
            .find(|l| l.contains(label))
            .unwrap_or_else(|| panic!("missing line for {label}"));
        assert!(
            line.contains(icon),
            "square icon {icon:?} not rendered for [{label}] in line: {line:?}"
        );
    }
}

#[test]
fn test_pretty_checkbox_circle_icons() {
    let stdout = run(&["--pretty-checkbox", "circle"], &checkbox_markdown());
    let expected = [
        ('\u{F0130}', "unchecked"),
        ('\u{F0133}', "done"),
        ('\u{F0376}', "canceled"),
        ('\u{F02D7}', "question"),
        ('\u{F0028}', "important"),
        ('\u{F0AA2}', "in progress"),
        ('\u{F0AA1}', "alt progress"),
        ('\u{F0AA0}', "backslash state"),
    ];
    for (icon, label) in expected {
        let line = stdout
            .lines()
            .find(|l| l.contains(label))
            .unwrap_or_else(|| panic!("missing line for {label}"));
        assert!(
            line.contains(icon),
            "circle icon not rendered for [{label}] in line: {line:?}"
        );
    }
}

#[test]
fn test_custom_checkbox_overrides_default() {
    // Override the unchecked icon and confirm it replaces the default.
    let md = "- [ ] overridden\n";
    let stdout = run(
        &[
            "--pretty-checkbox",
            "square",
            "--custom-checkbox",
            " :\u{F0026}",
        ],
        md,
    );
    let line = stdout.lines().find(|l| l.contains("overridden")).unwrap();
    assert!(line.contains('\u{F0026}'), "override not applied: {line:?}");
    // The default square unchecked icon must NOT appear anymore.
    assert!(!line.contains('\u{F0130}'), "default icon leaked: {line:?}");
}

#[test]
fn test_custom_checkbox_adds_new_state() {
    let md = "- [*] starred\n";
    let stdout = run(
        &[
            "--pretty-checkbox",
            "square",
            "--custom-checkbox",
            "*:\u{F078B}",
        ],
        md,
    );
    let line = stdout.lines().find(|l| l.contains("starred")).unwrap();
    assert!(
        line.contains('\u{F078B}'),
        "new state not rendered: {line:?}"
    );
}

#[test]
fn test_custom_checkbox_ignored_without_pretty() {
    // Without --pretty-checkbox, custom overrides must have no effect:
    // `[*]` stays a literal marker, `[x]` stays `[✓]`.
    let md = "- [*] starred\n- [x] done\n";
    let stdout = run(&["--custom-checkbox", "*:\u{F078B}"], md);
    let starred = stdout.lines().find(|l| l.contains("starred")).unwrap();
    assert!(
        starred.contains("[*]"),
        "custom state should stay literal without pretty mode: {starred:?}"
    );
    let done = stdout.lines().find(|l| l.contains("done")).unwrap();
    assert!(
        done.contains("[✓]"),
        "default checked marker changed: {done:?}"
    );
}

#[test]
fn test_backslash_checkbox_both_writings() {
    // Both `- [\]` (single backslash) and `- [\\]` (escaped) must render the icon.
    let md = "- [\\] single\n- [\\\\] double\n";
    let stdout = run(&["--pretty-checkbox", "square"], md);
    let single = stdout.lines().find(|l| l.contains("single")).unwrap();
    let double = stdout.lines().find(|l| l.contains("double")).unwrap();
    assert!(
        single.contains('\u{F0856}'),
        "single backslash not normalized: {single:?}"
    );
    assert!(
        double.contains('\u{F0856}'),
        "double backslash not rendered: {double:?}"
    );
}

#[test]
fn test_default_checkbox_unchanged_without_pretty() {
    // Backward compatibility: no flag -> `[ ]`, `[✓]`, literal `[c]`.
    let stdout = run(&[], &checkbox_markdown());
    let unchecked = stdout.lines().find(|l| l.contains("unchecked")).unwrap();
    assert!(unchecked.contains("[ ]"));
    let done = stdout.lines().find(|l| l.contains("done")).unwrap();
    assert!(done.contains("[✓]"));
    let canceled = stdout.lines().find(|l| l.contains("canceled")).unwrap();
    assert!(canceled.contains("[-]"));
}

#[test]
fn test_custom_checkbox_color_override() {
    // Color override: yellow for [*], custom RGB for [!]
    let md = "- [*] starred\n- [!] important\n";
    let stdout = run_with_colors(
        &[
            "--pretty-checkbox",
            "square",
            "--custom-checkbox",
            "*:\u{F078B}:yellow;!:\u{F0027}:128,1,1",
        ],
        md,
    );
    let starred = stdout.lines().find(|l| l.contains("starred")).unwrap();
    assert!(
        starred.contains("\x1b[33m")
            || starred.contains("\x1b[93m")
            || starred.contains("\x1b[38;5;3m"),
        "yellow color not applied to [*]: {starred:?}"
    );

    let important = stdout.lines().find(|l| l.contains("important")).unwrap();
    // RGB 128,1,1 = 38;2;128;1;1
    assert!(
        important.contains("\x1b[38;2;128;1;1m"),
        "RGB color not applied to [!]: {important:?}"
    );
}

#[test]
fn test_custom_checkbox_hex_color() {
    let md = "- [ ] test\n";
    let stdout = run_with_colors(
        &[
            "--pretty-checkbox",
            "square",
            "--custom-checkbox",
            " :\u{F0131}:#ff5500",
        ],
        md,
    );
    let line = stdout.lines().find(|l| l.contains("test")).unwrap();
    assert!(
        line.contains("\x1b[38;2;255;85;0m"),
        "hex color not applied: {line:?}"
    );
}

#[test]
fn test_custom_checkbox_no_color_still_works() {
    // Without color part, should still render the icon with default color
    let md = "- [*] starred\n";
    let stdout = run(
        &[
            "--pretty-checkbox",
            "square",
            "--custom-checkbox",
            "*:\u{F078B}",
        ],
        md,
    );
    let starred = stdout.lines().find(|l| l.contains("starred")).unwrap();
    assert!(
        starred.contains('\u{F078B}'),
        "icon without color should still render: {starred:?}"
    );
}

#[test]
fn test_pretty_checkbox_nested_indent() {
    // Nested checkboxes must preserve list-level indentation.
    let md = "- [ ] top\n  - [x] child\n    - [-] deep\n  - [?] back\n";
    let stdout = run(&["--pretty-checkbox", "square"], md);
    let lines: Vec<&str> = stdout.lines().collect();

    // top: 1 leading space (content_indent=0, list level 0, but icon replaces "- ")
    let top = lines.iter().find(|l| l.contains("top")).unwrap();
    let top_indent = top.len() - top.trim_start().len();
    // child: 3 leading spaces (list level 1 = 2 spaces + icon shift)
    let child = lines.iter().find(|l| l.contains("child")).unwrap();
    let child_indent = child.len() - child.trim_start().len();
    // deep: 5 leading spaces (list level 2 = 4 spaces + icon shift)
    let deep = lines.iter().find(|l| l.contains("deep")).unwrap();
    let deep_indent = deep.len() - deep.trim_start().len();

    assert!(
        top_indent < child_indent,
        "child should be more indented than top: top={top_indent} child={child_indent}"
    );
    assert!(
        child_indent < deep_indent,
        "deep should be more indented than child: child={child_indent} deep={deep_indent}"
    );
    // back should be at same level as child (list level 1)
    let back = lines.iter().find(|l| l.contains("back")).unwrap();
    let back_indent = back.len() - back.trim_start().len();
    assert_eq!(
        back_indent, child_indent,
        "back should match child indent: child={child_indent} back={back_indent}"
    );
}

#[test]
fn test_pretty_checkbox_heading_indent() {
    // Checkboxes under H2 should have +1 content indent vs H1.
    let md = "# H1\n\n- [ ] under h1\n\n## H2\n\n- [ ] under h2\n";
    let stdout = run(&["--pretty-checkbox", "square"], md);
    let h1_line = stdout.lines().find(|l| l.contains("under h1")).unwrap();
    let h2_line = stdout.lines().find(|l| l.contains("under h2")).unwrap();
    let h1_indent = h1_line.len() - h1_line.trim_start().len();
    let h2_indent = h2_line.len() - h2_line.trim_start().len();
    assert!(
        h2_indent > h1_indent,
        "H2 checkbox should be more indented than H1: h1={h1_indent} h2={h2_indent}"
    );
}

#[test]
fn test_pretty_checkbox_bullet_removed_not_regular_items() {
    // Pretty mode removes "-" only for checkbox items, not regular list items.
    let md = "- [ ] checkbox item\n- regular item\n";
    let stdout = run(&["--pretty-checkbox", "square"], md);
    let checkbox_line = stdout
        .lines()
        .find(|l| l.contains("checkbox item"))
        .unwrap();
    let regular_line = stdout.lines().find(|l| l.contains("regular item")).unwrap();
    // Checkbox line must NOT contain "- " prefix (bullet removed)
    let checkbox_stripped = checkbox_line.trim_start();
    assert!(
        !checkbox_stripped.starts_with("- "),
        "bullet should be removed for checkbox: {checkbox_line:?}"
    );
    // Regular item must still have "- " prefix
    let regular_stripped = regular_line.trim_start();
    assert!(
        regular_stripped.starts_with("- "),
        "bullet should remain for regular items: {regular_line:?}"
    );
}

#[test]
fn test_custom_checkbox_color_only_existing_state() {
    // `?:yellow` — color-only override for existing [?], icon stays default.
    let md = "- [?] question\n";
    let stdout = run_with_colors(&["--pretty-checkbox", "square", "-B", "?:yellow"], md);
    let line = stdout.lines().find(|l| l.contains("question")).unwrap();
    // Default square [?] icon should still be present.
    assert!(
        line.contains('\u{F078B}'),
        "default icon should remain for color-only override: {line:?}"
    );
    // Yellow color should be applied.
    assert!(
        line.contains("\x1b[93m") || line.contains("\x1b[33m") || line.contains("\x1b[38;5;3m"),
        "yellow color not applied for color-only override: {line:?}"
    );
}

#[test]
fn test_custom_checkbox_color_only_new_state() {
    // `*:yellow` — new [*] state with color only, no icon specified.
    // Should use the default unchecked icon + yellow color.
    let md = "- [*] starred\n";
    let stdout = run_with_colors(&["--pretty-checkbox", "square", "-B", "*:yellow"], md);
    let line = stdout.lines().find(|l| l.contains("starred")).unwrap();
    // Should use the default unchecked icon (F0131 for square).
    assert!(
        line.contains('\u{F0131}'),
        "new state with no icon should use default unchecked icon: {line:?}"
    );
    // Yellow color should be applied.
    assert!(
        line.contains("\x1b[93m") || line.contains("\x1b[33m") || line.contains("\x1b[38;5;3m"),
        "yellow color not applied for new state: {line:?}"
    );
}

#[test]
fn test_custom_checkbox_icon_and_color_together() {
    // `*:icon:color` — full override with icon + color.
    let md = "- [*] starred\n";
    let stdout = run_with_colors(
        &["--pretty-checkbox", "square", "-B", "*:\u{F078B}:red"],
        md,
    );
    let line = stdout.lines().find(|l| l.contains("starred")).unwrap();
    assert!(
        line.contains('\u{F078B}'),
        "custom icon should be used: {line:?}"
    );
    assert!(
        line.contains("\x1b[31m") || line.contains("\x1b[91m") || line.contains("\x1b[38;5;1m"),
        "red color not applied: {line:?}"
    );
}
