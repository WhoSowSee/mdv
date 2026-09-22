use super::*;

#[test]
fn short_flags_match_public_cli_contract() {
    let mut command = Cli::command();
    let _ = command.render_help();

    let mut actual = command
        .get_arguments()
        .filter_map(|arg| {
            arg.get_short()
                .map(|short| (arg.get_long().expect("short option has a long name"), short))
        })
        .collect::<Vec<_>>();
    actual.sort_unstable();

    let mut expected = vec![
        ("callout-style", 'C'),
        ("code-block-style", 'b'),
        ("code-line-numbers", 'K'),
        ("code-theme", 'T'),
        ("cols", 'c'),
        ("config-file", 'F'),
        ("definition-marker-style", 'D'),
        ("heading-layout", 'H'),
        ("help", 'h'),
        ("interactive", 'i'),
        ("line-numbers", 'N'),
        ("link-style", 'u'),
        ("list-style", 'L'),
        ("link-overflow", 'l'),
        ("margin", 'm'),
        ("no-config", 'n'),
        ("pager", 'p'),
        ("preset", 'P'),
        ("checkbox-style", 'x'),
        ("render-html", 'E'),
        ("reverse", 'r'),
        ("smart-indent", 'I'),
        ("table-borders", 'B'),
        ("table-smart-indent", 'S'),
        ("table-wrap", 'W'),
        ("theme", 't'),
        ("version", 'V'),
        ("wrap", 'w'),
    ];
    expected.sort_unstable();

    assert_eq!(actual, expected);
    assert!(
        command
            .get_arguments()
            .all(|arg| arg.get_all_short_aliases().is_none())
    );
}

#[test]
fn help_groups_options_in_task_oriented_order() {
    let help = Cli::command().render_long_help().to_string();
    let expected = [
        "Options:",
        "--help",
        "--version",
        "Output and flow:",
        "--pager",
        "--interactive",
        "--monitor",
        "--from",
        "--reverse",
        "--html",
        "--render-html",
        "--line-numbers",
        "--color",
        "[default: auto]",
        "--hide-comments",
        "--show-empty-elements",
        "Layout and wrapping:",
        "--cols",
        "--margin",
        "--tab-length",
        "--wrap",
        "--reflow",
        "--heading-layout",
        "--show-heading-markers",
        "--smart-indent",
        "--table-wrap",
        "--table-borders",
        "--table-smart-indent",
        "--block-spacing",
        "Themes and code:",
        "--theme",
        "--code-theme",
        "--theme-info",
        "--custom-theme",
        "--custom-code-theme",
        "--inline-style",
        "--code-block-style",
        "--math-block-style",
        "--code-line-numbers",
        "--custom-code-block",
        "--code-wrap-indent",
        "--syntaxes-dir",
        "--no-code-guessing",
        "Callouts and lists:",
        "--callout-style",
        "--custom-callout",
        "--checkbox-style",
        "--custom-checkbox",
        "--list-style",
        "--uniform-list-marker",
        "--custom-list",
        "--definition-marker-style",
        "Links and footnotes:",
        "--link-style",
        "--link-overflow",
        "--footnote-style",
        "--missing-footnote-style",
        "Configuration:",
        "--config-file",
        "--no-config",
        "--preset",
        "--preset-info",
        "--init-config",
    ];

    let mut offset = 0;
    for fragment in expected {
        let relative = help[offset..]
            .find(fragment)
            .unwrap_or_else(|| panic!("missing or misplaced help fragment: {fragment}"));
        offset += relative + fragment.len();
    }
}
