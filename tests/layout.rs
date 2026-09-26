use crate::support::mdv_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::{NamedTempFile, tempdir};

#[path = "layout/blockquotes.rs"]
mod blockquotes;
#[path = "layout/headings.rs"]
mod headings;
#[path = "layout/margins.rs"]
mod margins;
#[path = "layout/spacing.rs"]
mod spacing;

#[test]
fn horizontal_rule_styles_and_color() {
    let input = NamedTempFile::new().unwrap();
    fs::write(&input, "Before\n\n***\n\nMiddle\n\n---\n\nAfter\n").unwrap();

    for (args, expected) in [
        (vec![], format!("◈{}◈", "─".repeat(10))),
        (
            vec![
                "--horizontal-rule-style",
                "simple",
                "--custom-theme",
                "horizontal_rule=#123456",
            ],
            format!("\x1b[38;2;18;52;86m{}", "─".repeat(12)),
        ),
    ] {
        let output = mdv_cmd()
            .arg("--no-config")
            .args([
                "--color",
                "always",
                "--color-depth",
                "truecolor",
                "--cols",
                "12",
                "--heading-layout",
                "none",
            ])
            .args(args)
            .arg(input.path())
            .output()
            .expect("mdv renders horizontal rules");
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(
            stdout
                .lines()
                .filter(|line| line.starts_with(&expected))
                .count(),
            2,
            "{stdout}"
        );
    }
}

#[test]
fn horizontal_rule_style_loads_from_config_and_cli_overrides_it() {
    let config_dir = tempdir().unwrap();
    fs::write(
        config_dir.path().join("config.yaml"),
        "horizontal_rule_style: simple\n",
    )
    .unwrap();
    let input = NamedTempFile::new().unwrap();
    fs::write(&input, "Before\n\n***\n\nAfter\n").unwrap();

    for (args, expected) in [
        (vec![], "─".repeat(12)),
        (
            vec!["--horizontal-rule-style", "pretty"],
            format!("◈{}◈", "─".repeat(10)),
        ),
    ] {
        let output = mdv_cmd()
            .arg("--config-file")
            .arg(config_dir.path())
            .args([
                "--color",
                "never",
                "--cols",
                "12",
                "--heading-layout",
                "none",
            ])
            .args(args)
            .arg(input.path())
            .output()
            .expect("mdv renders the configured rule");
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.lines().any(|line| line == expected), "{stdout}");
    }
}
