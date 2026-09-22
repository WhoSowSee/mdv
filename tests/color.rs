use crate::support::mdv_cmd;
use assert_cmd::Command;
use mdv::utils::strip_ansi;
use std::{fs, process::Stdio};
use tempfile::{NamedTempFile, TempDir};

const DOCUMENT: &str = include_str!("files/color-mode.md");

const RENDER_ARGS: &[&str] = &[
    "--cols",
    "80",
    "--render-html",
    "--front-matter",
    "panel",
    "--line-numbers",
    "source;separator",
    "--code-line-numbers",
    "source;separator",
    "--math-block-style",
    "pretty",
    "--code-block-style",
    "pretty",
    "--custom-theme",
    "h1=#010203;link=#040506",
];

fn render(command: &mut Command, markdown: &str) -> String {
    let assertion = command.write_stdin(markdown).assert().success();
    String::from_utf8(assertion.get_output().stdout.clone()).unwrap()
}

fn error(command: &mut Command) -> String {
    let assertion = command
        .arg("-")
        .write_stdin("# Heading\n")
        .assert()
        .failure();
    assert!(assertion.get_output().stdout.is_empty());
    let error = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(!error.contains('\x1b'), "{error}");
    error
}

fn configured_command(config: &TempDir) -> Command {
    let mut command = mdv_cmd();
    command.arg("--config-file").arg(config.path());
    command
}

#[test]
fn color_depth_limits_mixed_output_without_changing_text_or_links() {
    let mut visible = None;
    let extended = regex::Regex::new(r"\x1b\[(?:\d+;)*(?:38|48|58);(?:2|5);").unwrap();
    let rgb = regex::Regex::new(r"\x1b\[(?:\d+;)*(?:38|48|58);2;").unwrap();
    for depth in ["truecolor", "256", "16"] {
        let output = render(
            mdv_cmd().args(RENDER_ARGS).args([
                "--color=always",
                "--table-borders",
                "--theme=tokyonight",
                "--color-depth",
                depth,
            ]),
            DOCUMENT,
        );
        assert!(output.contains("\x1b]8;;https://example.com"));
        match depth {
            "16" => {
                assert!(!extended.is_match(&output));
                assert!(
                    output.contains("\x1b[94m") && output.contains("\x1b[92m"),
                    "blue and green syntax colors must remain distinct"
                );
            }
            "256" => {
                assert!(!rgb.is_match(&output));
                assert!(extended.is_match(&output));
            }
            _ => assert!(rgb.is_match(&output)),
        }
        let text = strip_ansi(&output);
        assert_eq!(&text, visible.get_or_insert_with(|| text.clone()));
    }
}

#[test]
fn pipe_modes_preserve_geometry_and_ignore_unrelated_color_environment() {
    let mut visible = String::new();
    for (mode, old_value) in [
        ("always", "True"),
        ("", "False"),
        ("auto", "False"),
        ("never", "False"),
    ] {
        let mut command = mdv_cmd();
        for name in ["NO_COLOR", "CLICOLOR", "CLICOLOR_FORCE", "FORCE_COLOR"] {
            command.env(name, "1");
        }
        command
            .env("MDV_NO_COLOR", old_value)
            .env("MDV_NO_COLORS", old_value)
            .args(RENDER_ARGS);
        if !mode.is_empty() {
            command.args(["--color", mode]);
        }
        if mode == "never" {
            command.arg("--color-depth=16");
        }
        let output = render(command.arg("-"), DOCUMENT);
        if mode == "always" {
            assert!(output.contains("\x1b[38;2;1;2;3m"));
            assert!(output.contains("\x1b]8;;https://example.com"));
            visible = strip_ansi(&output);
        } else {
            assert!(!output.contains('\x1b'), "{mode}: {output}");
            assert_eq!(output, visible, "{mode}");
        }
    }
}

#[test]
fn file_redirection_is_plain_by_default() {
    let source = NamedTempFile::new().unwrap();
    let output = NamedTempFile::new().unwrap();
    let config = TempDir::new().unwrap();
    fs::write(source.path(), DOCUMENT).unwrap();
    let status = std::process::Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
        .env_remove("MDV_COLOR")
        .env("MDV_CONFIG_PATH", config.path())
        .arg(source.path())
        .stdout(Stdio::from(output.reopen().unwrap()))
        .status()
        .unwrap();
    assert!(status.success());
    let output = fs::read_to_string(output.path()).unwrap();
    assert!(!output.contains('\x1b'));
    assert!(output.contains("Heading"));
}

#[test]
fn color_settings_reach_the_renderer_and_explicit_cli_wins() {
    let config = TempDir::new().unwrap();
    fs::write(config.path().join("config.yaml"), "color: always\n").unwrap();
    fs::create_dir(config.path().join("presets")).unwrap();
    fs::write(
        config.path().join("presets/plain.yaml"),
        "name: plain\ncolor: never\n",
    )
    .unwrap();
    let mut command = configured_command(&config);
    command
        .args(["--preset", "plain"])
        .env("MDV_COLOR", "always");
    assert!(render(command.arg("-"), "# Heading").contains('\x1b'));
    command.args(["--color", "never"]);
    assert!(!render(&mut command, "# Heading").contains('\x1b'));
}

#[test]
fn loader_errors_are_reported_without_fallback_or_override_masking() {
    let config = TempDir::new().unwrap();
    fs::write(config.path().join("config.yaml"), "color: INVALID\n").unwrap();
    fs::write(config.path().join("config.yml"), "color: always\n").unwrap();
    let invalid = error(configured_command(&config).args(["--color", "always"]));
    assert!(invalid.contains("config.yaml") && invalid.contains("color"));
    assert!(
        !render(
            configured_command(&config).args(["--no-config", "-"]),
            "# Heading"
        )
        .contains('\x1b')
    );
    fs::remove_file(config.path().join("config.yaml")).unwrap();
    assert!(render(configured_command(&config).arg("-"), "# Heading").contains('\x1b'));
    let invalid_env = error(
        mdv_cmd()
            .env("MDV_COLOR", "invalid")
            .args(["--color", "never"]),
    );
    assert!(invalid_env.contains("MDV_COLOR") && invalid_env.contains("auto, always, never"));
    fs::write(
        config.path().join("config.yaml"),
        "color: auto\nno_colors: true\n",
    )
    .unwrap();
    assert!(error(&mut configured_command(&config)).contains("was removed; use 'color:"));
    fs::create_dir(config.path().join("presets")).unwrap();
    fs::write(
        config.path().join("presets/old.yaml"),
        "name: old\nno_colors: true\n",
    )
    .unwrap();
    assert!(
        error(configured_command(&config).args(["--no-config", "--preset", "old"]))
            .contains("was removed; use 'color:")
    );
}

#[test]
fn pager_pipe_metadata_and_html_follow_output_contract() {
    for mode in ["auto", "always", "never"] {
        let output = render(
            mdv_cmd().args(["--color", mode, "--pager", "-"]),
            "# Heading",
        );
        assert_eq!(output.contains('\x1b'), mode == "always");
    }
    assert!(
        error(mdv_cmd().args(["--color", "always", "--interactive"]))
            .contains("requires a terminal")
    );
    for flag in ["--theme-info", "--preset-info"] {
        assert!(!render(mdv_cmd().args(["--color", "never", flag]), "").contains('\x1b'));
    }
    let html = render(
        mdv_cmd().args(["--color", "always", "--html", "-"]),
        DOCUMENT,
    );
    assert!(!html.contains('\x1b'));
    assert!(html.contains("<h1>Heading</h1>"));
}

#[test]
fn fast_commands_are_plain_and_do_not_load_color_settings() {
    let config = TempDir::new().unwrap();
    fs::write(config.path().join("config.yaml"), "no_colors: true\n").unwrap();
    for flag in ["--help", "--version"] {
        let output = render(
            configured_command(&config)
                .env("MDV_COLOR", "invalid")
                .args(["--color", "always", flag]),
            "",
        );
        assert!(!output.contains('\x1b'));
    }
    mdv_cmd()
        .env("MDV_COLOR", "invalid")
        .arg("help")
        .assert()
        .failure()
        .stderr(predicates::str::contains("MDV_COLOR"));
    let destination = TempDir::new().unwrap();
    mdv_cmd()
        .env("MDV_COLOR", "invalid")
        .arg("--init-config")
        .arg(destination.path())
        .assert()
        .success();
    let template = fs::read_to_string(destination.path().join("config.yaml")).unwrap();
    assert!(template.contains("color: auto"));
    assert!(!template.contains("no_colors"));
}
