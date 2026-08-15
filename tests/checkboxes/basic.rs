use super::*;

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
fn test_default_checkbox_bullet_removed_not_regular_or_ordered_items() {
    let md = "- [ ] unchecked\n- [x] done\n- [?] question\n- regular\n1. [ ] ordered\n";
    let stdout = run(&[], md);

    for label in ["unchecked", "done", "question"] {
        let line = stdout.lines().find(|line| line.contains(label)).unwrap();
        assert!(
            !line.trim_start().starts_with("- "),
            "bullet should be removed for checkbox item: {line:?}"
        );
    }

    let regular = stdout
        .lines()
        .find(|line| line.contains("regular"))
        .unwrap();
    assert!(regular.trim_start().starts_with("- "));

    let ordered = stdout
        .lines()
        .find(|line| line.contains("ordered"))
        .unwrap();
    assert!(ordered.trim_start().starts_with("1. "));
}

#[test]
fn test_unknown_checkbox_states_render_literal_without_list_marker() {
    let md = "- [*] starred\n- [z] custom\n";
    let modes: &[&[&str]] = &[
        &[],
        &["--pretty-checkbox", "square"],
        &[
            "--pretty-checkbox",
            "square",
            "--pretty-list",
            "type:unicode;size:small",
        ],
    ];

    for args in modes {
        let stdout = run(args, md);
        let lines: Vec<_> = stdout
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect();
        assert_eq!(
            lines,
            ["[*] starred", "[z] custom"],
            "unexpected checkbox rendering for {args:?}"
        );
    }
}
