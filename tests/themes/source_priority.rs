use super::*;

#[test]
fn source_priority_precedes_specificity_and_preserves_icons() {
    let directory = fixture(&[
        (
            "config.yaml",
            "theme: common\ncustom_callout: {note: {icon: N, color: green}}\ncheckbox_style: square\ncustom_checkbox: {x: {icon: X, color: green}}\nlist_style: type:unicode;size:small\ncustom_list: {1: {icon: L, color: green}}\n",
        ),
        (
            "themes/common.yaml",
            "name: common\nextends: terminal\ncallout: {label: red}\n",
        ),
        (
            "presets/blue.yaml",
            "name: blue\ncustom_theme: {callout: {label: blue}, todo: {checked: blue}, list: {unordered: blue}}\n",
        ),
    ]);
    let path = directory.path().to_str().unwrap();
    let markdown = "> [!note]\n> Body\n\n- Item\n- [x] Done\n";
    for mut args in [
        vec!["--custom-theme=callout:label=blue;todo:checked=blue;list:unordered=blue"],
        vec!["--preset=blue"],
    ] {
        args.extend([
            "--callout-style=simple:show-icons",
            "--custom-callout=note:icon=M",
            "--custom-checkbox=x:Z",
            "--custom-list=1:V",
        ]);
        let output = configured_render(markdown, path, &args);
        for expected in ["\x1b[94m\x1b[1mM  Note", "\x1b[94mV ", "\x1b[94mZ"] {
            assert!(output.contains(expected), "{args:?}: {output}");
        }
    }
    let output = configured_render(
        markdown,
        path,
        &[
            "--callout-style=simple:show-icons",
            "--custom-theme=callout:label=blue",
            "--custom-callout=note:color=magenta",
        ],
    );
    assert!(output.contains("\x1b[95m\x1b[1mN  Note"), "{output}");
    fs::write(
        directory.path().join("config.yaml"),
        "theme: common\ncustom_callout: {note: {color: green}}\n",
    )
    .unwrap();
    let output = configured_render(
        "> [!note]\n> Body\n\n> [!tip]\n> Other\n",
        path,
        &[
            "--callout-style=simple",
            "--custom-theme=callout:palette:note=blue",
        ],
    );
    for expected in ["\x1b[94m\x1b[1m[Note]", "\x1b[91m\x1b[1m[Tip]"] {
        assert!(output.contains(expected), "{output}");
    }
}

#[test]
fn selected_code_theme_keeps_unmodified_colors_and_respects_sources() {
    fn sample_line(output: &str) -> &str {
        output
            .lines()
            .find(|line| strip_ansi(line).contains("sample"))
            .expect("string line")
    }
    let directory = fixture(&[(
        "config.yaml",
        "custom_theme: {syntax: {number: green}}\ncustom_code_theme: {number: yellow}\n",
    )]);
    let path = directory.path().to_str().unwrap();
    let markdown = "```rust\nlet count = 42;\nlet text = \"sample\";\n```\n";
    for theme in ["nord", "InspiredGitHub"] {
        let base = render(markdown, &["--code-theme", theme]);
        for main in [None, Some("syntax:number=blue")] {
            let mut args = vec!["--code-theme", theme];
            if let Some(main) = main {
                args.extend(["--custom-theme", main]);
            }
            assert_eq!(configured_render(markdown, path, &args), base, "{args:?}");
        }
        let overridden = configured_render(
            markdown,
            path,
            &[
                "--code-theme",
                theme,
                "--custom-code-theme=syntax:number=blue",
            ],
        );
        assert!(overridden.contains("\x1b[94m42"), "{theme}: {overridden}");
        assert_eq!(sample_line(&overridden), sample_line(&base), "{theme}");
    }
    fs::write(directory.path().join("config.yaml"), "code_theme: nord\ncustom_theme: {syntax: {number: green}}\ncustom_code_theme: {number: yellow}\n").unwrap();
    for (args, expected) in [
        (&[][..], "\x1b[93m42"),
        (&["--custom-theme=syntax:number=blue"][..], "\x1b[94m42"),
    ] {
        let output = configured_render(markdown, path, args);
        assert!(output.contains(expected), "{args:?}: {output}");
    }
}
