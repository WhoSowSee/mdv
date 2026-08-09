use assert_cmd::Command;
use mdv::utils::{display_width, strip_ansi};
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn renders_definition_lists_in_plain_and_pretty_modes() {
    let markdown = NamedTempFile::new().unwrap();
    fs::write(
        &markdown,
        "Empty Term\n:\n\nTerm One\n: First definition line.\n  Soft continuation.\n\n  Second paragraph in the same definition.\n: Second definition line for additional context.\n\nTerm Two\n: Separate definition entry with its own description.\n",
    )
    .unwrap();

    for (style, prefix) in [
        (None, "  "),
        (Some("unicode"), "🠶 "),
        (Some("nerd-font"), "\u{f0315} "),
    ] {
        let mut command = Command::new(assert_cmd::cargo::cargo_bin!("mdv"));
        command.args([
            "--no-colors",
            "--no-config",
            "--cols",
            "44",
            "--wrap",
            "word",
        ]);
        if let Some(style) = style {
            command.args(["--pretty-definition", style]);
        }

        let output = command.arg(markdown.path()).output().unwrap();
        assert!(output.status.success(), "style={style:?}");
        let clean = strip_ansi(&String::from_utf8(output.stdout).unwrap());
        let normalized = clean
            .lines()
            .map(str::trim_end)
            .collect::<Vec<_>>()
            .join("\n");
        let continuation_indent = if style.is_none() { "  " } else { "" };
        assert!(
            !normalized.contains(&format!("Empty Term\n{prefix}"))
                && normalized.contains(&format!(
                    "Term One\n{prefix}First definition line.\n{continuation_indent}Soft continuation.\n\n{continuation_indent}Second paragraph in the same definition."
                ))
                && normalized.contains(&format!("\n\n{prefix}Second definition line"))
                && normalized.contains(&format!(
                    "\n\nTerm Two\n{prefix}Separate definition entry"
                )),
            "style={style:?}, stdout:\n{clean}"
        );
        assert!(
            clean.lines().all(|line| display_width(line) <= 44),
            "style={style:?}, stdout:\n{clean}"
        );
    }
}
