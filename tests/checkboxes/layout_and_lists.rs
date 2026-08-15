use super::*;

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
fn test_pretty_list_and_pretty_checkbox_coexist() {
    let md = "- [ ] checkbox item\n- regular item\n";
    let stdout = run(
        &[
            "--pretty-list",
            "type:nerd-font;size:large",
            "--pretty-checkbox",
            "square",
        ],
        md,
    );
    let checkbox_line = stdout
        .lines()
        .find(|l| l.contains("checkbox item"))
        .unwrap();
    let regular_line = stdout.lines().find(|l| l.contains("regular item")).unwrap();
    assert!(
        checkbox_line.contains('\u{F0131}'),
        "checkbox icon missing: {checkbox_line:?}"
    );
    assert!(
        !checkbox_line.contains('\u{F444}'),
        "pretty-list bullet icon should be stripped for checkbox item: {checkbox_line:?}"
    );
    assert!(
        regular_line.contains('\u{F444}'),
        "pretty-list bullet icon should remain for regular item: {regular_line:?}"
    );
}

#[test]
fn test_pretty_list_unicode_icons() {
    let stdout = run(
        &["--pretty-list", "type:unicode;size:small"],
        nested_list_markdown(),
    );

    for icon in ['⦁', '▪', '⚬', '▫'] {
        assert!(
            stdout.contains(icon),
            "missing Unicode icon {icon:?}: {stdout:?}"
        );
    }
    assert_eq!(stdout.matches('▫').count(), 2);
}

#[test]
fn test_uniform_list_marker_accepts_level_or_icon() {
    let from_level = run(
        &[
            "--pretty-list",
            "type:unicode;size:large",
            "--uniform-list-marker",
            "level:2",
        ],
        nested_list_markdown(),
    );
    assert_eq!(from_level.matches('▪').count(), 5);
}
