use super::*;

#[test]
fn optional_colors_preserve_omission_null_reset_and_legacy_rejection() {
    let fields = [
        "background",
        "emphasis:background",
        "strong:background",
        "strong_emphasis:background",
        "code:background",
        "strikethrough:background",
    ];
    let backgrounds = |value: &str| {
        fields
            .map(|field| format!("{}: {value}\n", field.replace(':', ":\n  ")))
            .join("")
    };
    let directory = fixture(&[
        (
            "themes/a-parent.yaml",
            &format!(
                "name: parent\nextends: terminal\n{}math: {{border: red}}\nfront_matter: {{border: blue}}\n",
                backgrounds("red")
            ),
        ),
        (
            "themes/b-inherited.yaml",
            "name: inherited\nextends: parent\n",
        ),
        (
            "themes/c-cleared.yaml",
            &format!(
                "name: cleared\nextends: parent\n{}math: {{border: null}}\nfront_matter: {{border: null}}\n",
                backgrounds("null")
            ),
        ),
        (
            "themes/d-reset.yaml",
            &format!(
                "name: reset-backgrounds\nextends: parent\n{}",
                backgrounds("reset")
            ),
        ),
    ]);
    let path = directory.path().to_str().unwrap();
    let markdown = "---\ntitle: Demo\n---\n\nplain *italic* **bold** ***both*** `code` ~~strike~~\n\n$$x^2$$\n";
    let common = ["--front-matter", "panel", "--math-block-style", "pretty"];
    let from_theme = |name: &str| {
        let mut args = common.to_vec();
        args.extend(["--config-file", path, "--theme", name]);
        render(markdown, &args)
    };
    let cleared = from_theme("cleared");
    let mut args = common.to_vec();
    args.extend([
        "--custom-theme",
        "math:border=null;front_matter:border=null",
    ]);
    assert_eq!(cleared, render(markdown, &args));
    let borders: Vec<_> = cleared
        .lines()
        .filter(|line| strip_ansi(line).starts_with('╭'))
        .collect();
    assert_eq!(borders.len(), 2, "{cleared}");
    assert!(
        borders.iter().all(|line| line.starts_with('╭')),
        "{cleared}"
    );
    assert_ne!(from_theme("inherited"), cleared);
    let clears = format!(
        "{};math:border=null;front_matter:border=null",
        fields.map(|field| format!("{field}=null")).join(";")
    );
    let mut args = common.to_vec();
    args.extend([
        "--config-file",
        path,
        "--theme",
        "inherited",
        "--custom-theme",
        &clears,
    ]);
    assert_eq!(render(markdown, &args), cleared);
    let reset = from_theme("reset-backgrounds");
    assert!(reset.contains("\x1b[49m"), "{reset}");
    mdv_cmd()
        .args(["--no-config", "--custom-theme", "border=red", "-"])
        .write_stdin("> Quote\n")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Unknown key for custom theme: 'border'",
        ));
}
