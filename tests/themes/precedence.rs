use super::*;

#[test]
fn theme_overrides_merge_and_clear_by_source() {
    let preset =
        "custom_theme: {quote: cyan, syntax: {number: magenta, keyword: red, string: cyan}}\n";
    let directory = fixture(&[
        (
            "themes/priority.yaml",
            "name: priority\nextends: terminal\nquote: red\nmath: {border: red}\nsyntax: {number: red, keyword: cyan, string: cyan}\n",
        ),
        (
            "config.yaml",
            "theme: priority\ncustom_theme: {quote: green, math: {border: green}, syntax: {number: green, string: yellow}}\ncustom_code_theme: {number: yellow, keyword: magenta}\n",
        ),
        ("presets/main.yaml", &format!("name: main\n{preset}")),
        (
            "presets/both.yaml",
            &format!("name: both\n{preset}custom_code_theme: {{number: cyan}}\n"),
        ),
    ]);
    let cases: &[(&[&str], [&str; 5])] = &[
        (&[], ["green", "green", "yellow", "magenta", "yellow"]),
        (
            &["--custom-theme=syntax:number=blue"],
            ["green", "green", "blue", "magenta", "yellow"],
        ),
        (
            &["--custom-code-theme=syntax:number=blue"],
            ["green", "green", "blue", "magenta", "yellow"],
        ),
        (
            &[
                "--custom-theme=syntax:number=blue",
                "--custom-code-theme=number=cyan",
            ],
            ["green", "green", "cyan", "magenta", "yellow"],
        ),
        (
            &["--custom-theme=quote=blue;math:border=null"],
            ["blue", "null", "yellow", "magenta", "yellow"],
        ),
        (
            &["--preset=main"],
            ["cyan", "green", "magenta", "red", "cyan"],
        ),
        (&["--preset=both"], ["cyan", "green", "cyan", "red", "cyan"]),
        (
            &["--preset=both", "--custom-theme=syntax:number=blue"],
            ["cyan", "green", "blue", "red", "cyan"],
        ),
        (
            &["--preset=both", "--custom-code-theme=number=blue"],
            ["cyan", "green", "blue", "red", "cyan"],
        ),
    ];
    let markdown = "> Quote\n\n```rust\nlet count = 42;\nlet text = \"sample\";\n```\n\n$$x^2$$\n";
    let path = directory.path().to_str().unwrap();
    for &(args, [quote, border, number, keyword, string]) in cases {
        let mut args = args.to_vec();
        args.extend(["--math-block-style=pretty"]);
        let colors = format!(
            "quote={quote};math:border={border};syntax:number={number};syntax:keyword={keyword};syntax:string={string}"
        );
        let expected = render(
            markdown,
            &[
                "--config-file",
                path,
                "--theme",
                "priority",
                "--math-block-style=pretty",
                "--custom-theme",
                &colors,
            ],
        );
        assert_eq!(
            configured_render(markdown, path, &args),
            expected,
            "{args:?}"
        );
    }
    fs::write(directory.path().join("config.yaml"), "custom_theme: {quote: green, syntax: {number: blue}}\ncustom_code_theme: {number: yellow}\n").unwrap();
    for (setting, colors) in [
        ("custom_theme", "syntax:number=yellow"),
        ("custom_code_theme", "quote=green;syntax:number=blue"),
    ] {
        for value in ["{}", "null"] {
            fs::write(
                directory.path().join("presets/clear.yaml"),
                format!("name: clear\n{setting}: {value}\n"),
            )
            .unwrap();
            let actual = configured_render(markdown, path, &["--preset=clear"]);
            assert_eq!(
                actual,
                render(markdown, &["--custom-theme", colors]),
                "{setting}: {value}"
            );
        }
    }
}
