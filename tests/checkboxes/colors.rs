use super::*;

#[test]
fn test_empty_checkbox_visibility_follows_show_empty_elements() {
    let md = "- [ ]\n- [x]\n- [-]\n- [*]\n- [z]\n";
    let modes: &[(&[&str], &[&str])] = &[
        (&[], &["[ ]", "[✓]", "[-]", "[*]", "[z]"]),
        (
            &["--pretty-checkbox", "square"],
            &["\u{F0131}", "\u{F0132}", "\u{F0375}", "[*]", "[z]"],
        ),
        (
            &[
                "--pretty-checkbox",
                "square",
                "--pretty-list",
                "type:unicode;size:small",
            ],
            &["\u{F0131}", "\u{F0132}", "\u{F0375}", "[*]", "[z]"],
        ),
    ];

    for (args, expected) in modes {
        let hidden = run(args, md);
        assert!(
            hidden.trim().is_empty(),
            "empty checkboxes should be hidden for {args:?}: {hidden:?}"
        );

        let mut shown_args = args.to_vec();
        shown_args.push("--show-empty-elements");
        let shown = run(&shown_args, md);
        let lines: Vec<_> = shown
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect();
        assert_eq!(lines, *expected, "unexpected empty checkboxes for {args:?}");
    }
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
