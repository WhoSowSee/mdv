use crate::support::mdv_cmd;
use mdv::utils::strip_ansi;
use std::fs;
use tempfile::TempDir;

mod backgrounds;
mod precedence;
mod source_priority;

fn fixture(files: &[(&str, &str)]) -> TempDir {
    let directory = TempDir::new().unwrap();
    for (name, contents) in files {
        let path = directory.path().join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }
    directory
}

fn output(markdown: &str, config: Option<&str>, args: &[&str]) -> String {
    let mut command = mdv_cmd();
    command.args([
        "--color-depth=truecolor",
        "--cols=80",
        "--heading-layout=none",
    ]);
    if let Some(path) = config {
        command.args(["--config-file", path]);
    } else {
        command.arg("--no-config");
    }
    if !args.contains(&"--color") {
        command.args(["--color", "always"]);
    }
    let assertion = command
        .args(args)
        .arg("-")
        .write_stdin(markdown)
        .assert()
        .success();
    String::from_utf8(assertion.get_output().stdout.clone()).unwrap()
}

fn render(markdown: &str, args: &[&str]) -> String {
    output(markdown, None, args)
}
fn configured_render(markdown: &str, path: &str, args: &[&str]) -> String {
    output(markdown, Some(path), args)
}

#[test]
fn nested_theme_file_config_and_cli_produce_the_same_colors() {
    let colors = "line_number: {number: red, separator: darkgrey}\nsyntax: {number: magenta, keyword: blue}\ncode_block: {label: cyan}\n";
    let directory = fixture(&[
        (
            "themes/custom.yaml",
            &format!("name: custom\nextends: terminal\n{colors}"),
        ),
        (
            "config.yaml",
            &format!(
                "custom_theme:\n{}",
                colors
                    .lines()
                    .map(|line| format!("  {line}\n"))
                    .collect::<String>()
            ),
        ),
    ]);
    let path = directory.path().to_str().unwrap();
    let markdown = "```rust\nlet count = 42;\n```\n";
    let common = [
        "--line-numbers",
        "separator",
        "--code-block-style=pretty:show-name;show-icon",
        "--custom-code-block=rust:icon=R",
    ];
    let mut args = common.to_vec();
    args.extend(["--custom-theme=line_number:number=red;line_number:separator=darkgrey;syntax:number=magenta;syntax:keyword=blue;code_block:label=cyan"]);
    let expected = render(markdown, &args);
    let mut args = common.to_vec();
    args.extend(["--config-file", path, "--theme", "custom"]);
    assert_eq!(render(markdown, &args), expected);
    assert_eq!(configured_render(markdown, path, &common), expected);
    assert!(
        expected.contains("\x1b[95m42") && expected.contains("\x1b[91m1"),
        "{expected}"
    );
}

#[test]
fn labels_borders_lists_and_checkboxes_use_independent_colors() {
    let markdown = "---\ntitle: Demo\n---\n\n> Plain quote\n\n> [!note] Caption\n> Callout body\n\n```rust\nlet count = 42;\n```\n\n1. Ordered\n\n- Unordered\n- [ ] Open\n- [x] Done\n\n<details><summary>More</summary><p>Detail body</p></details>\n";
    for (style, border) in [("simple:show-icons", "┃"), ("pretty:show-icons", "╭")] {
        let mut args = vec![
            "--front-matter=panel",
            "--render-html",
            "--callout-style",
            style,
            "--custom-callout=note:icon=N",
            "--custom-code-block=rust:icon=R",
            "--code-block-style=pretty:show-name;show-icon",
        ];
        let mut plain_args = args.clone();
        plain_args.extend(["--color", "never"]);
        let plain = render(markdown, &plain_args);
        args.extend(["--custom-theme=callout:label=#010203;callout:border=#070809;code_block:label=#0a0b0c;code_block:border=#101112;list:ordered=#131415;list:unordered=#161718;todo:unchecked=#191a1b;todo:checked=#1c1d1e;details_border=#1f2021;front_matter:title=#222324;front_matter:border=#252627"]);
        let output = render(markdown, &args);
        for (color, text) in [
            ("10;11;12", "R Rust"),
            ("19;20;21", "1. "),
            ("22;23;24", "- "),
            ("25;26;27", "[ ]"),
            ("28;29;30", "[✓]"),
            ("31;32;33", "│"),
            ("37;38;39", ""),
            ("16;17;18", "╭"),
            ("7;8;9", border),
        ] {
            assert!(
                output.contains(&format!("\x1b[38;2;{color}m{text}")),
                "{style}: {color}/{text}: {output}"
            );
        }
        for (color, title) in [("1;2;3", "Caption"), ("34;35;36", "Properties")] {
            let prefix = regex::escape(&format!("\x1b[38;2;{color}m\x1b[1m"));
            assert!(
                regex::Regex::new(&format!("{prefix}\\S+ +{title}"))
                    .unwrap()
                    .is_match(&output),
                "{output}"
            );
        }
        assert_eq!(strip_ansi(&output), plain);
    }
}

#[test]
fn callout_palette_is_independent_of_quote_and_other_element_colors() {
    let markdown = [
        "note", "abstract", "info", "todo", "tip", "success", "question", "warning", "failure",
        "danger", "bug", "example", "quote",
    ]
    .map(|kind| format!("> [!{kind}]\n> Body\n"))
    .join("\n");
    let original = render(&markdown, &["--callout-style=simple"]);
    let changed = render(
        &markdown,
        &[
            "--callout-style=simple",
            "--custom-theme=quote=red;code:text=blue;table:header=magenta;list:unordered=yellow;h1=cyan;h2=red",
        ],
    );
    assert_eq!(original, changed);
    let overridden = render(
        &markdown,
        &[
            "--callout-style=simple",
            "--custom-theme=callout:palette:success=red",
        ],
    );
    assert!(
        overridden.contains("\x1b[91m\x1b[1m[Success]"),
        "{overridden}"
    );
}
