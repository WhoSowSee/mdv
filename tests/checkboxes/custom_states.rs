use super::*;

#[test]
fn test_custom_list_marker_stripped_for_checkbox() {
    let md = "- [ ] checkbox\n- regular\n";
    for icon in ["*", ">>>", "* *"] {
        let marker = format!("1:{icon}");
        let stdout = run(
            &[
                "--pretty-list",
                "type:nerd-font;size:large",
                "--custom-list",
                marker.as_str(),
                "--pretty-checkbox",
                "square",
            ],
            md,
        );
        let checkbox_line = stdout.lines().find(|l| l.contains("checkbox")).unwrap();
        let regular_line = stdout.lines().find(|l| l.contains("regular")).unwrap();
        assert!(
            !checkbox_line.contains(icon),
            "custom marker {icon:?} should be stripped for checkbox: {checkbox_line:?}"
        );
        assert!(
            regular_line.contains(icon),
            "custom marker {icon:?} should remain for regular: {regular_line:?}"
        );
    }
}

#[test]
fn test_custom_checkbox_color_only_existing_state() {
    // `?:yellow` — color-only override for existing [?], icon stays default.
    let md = "- [?] question\n";
    let stdout = run_with_colors(
        &[
            "--pretty-checkbox",
            "square",
            "--custom-checkbox",
            "?:yellow",
        ],
        md,
    );
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
    let stdout = run_with_colors(
        &[
            "--pretty-checkbox",
            "square",
            "--custom-checkbox",
            "*:yellow",
        ],
        md,
    );
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
        &[
            "--pretty-checkbox",
            "square",
            "--custom-checkbox",
            "*:\u{F078B}:red",
        ],
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
