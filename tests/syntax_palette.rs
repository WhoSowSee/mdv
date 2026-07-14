//! End-to-end checks that the default terminal theme follows the terminal palette.

use assert_cmd::Command;
use std::fs;
use tempfile::NamedTempFile;

fn mdv_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mdv"))
}

const BASH_SAMPLE: &str =
    "```bash\nomarchy-theme-install https://github.com/example/light-theme.git\n```";
const RUST_SAMPLE: &str = "```rust\nfn main() {}\n```";

const TRUECOLOR_WHITE: &str = "\x1b[38;2;255;255;255m";
const TERMINAL_RESET: &str = "\x1b[39m";
const BORDER_COLOR: &str = "\x1b[38;2;143;147;162m";

fn stdout(args: &[&str], content: &str) -> String {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, content).unwrap();
    let mut cmd = mdv_cmd();
    cmd.args(args).arg(temp_file.path());
    String::from_utf8_lossy(&cmd.assert().success().get_output().stdout).into_owned()
}

#[test]
fn default_theme_bash_block_has_no_truecolor_white() {
    let out = stdout(&["--no-config"], BASH_SAMPLE);
    assert!(
        !out.contains(TRUECOLOR_WHITE),
        "white truecolour leaked: {out:?}"
    );
    assert!(
        out.contains(TERMINAL_RESET),
        "must reset to terminal foreground: {out:?}"
    );
}

#[test]
fn default_theme_rust_uses_palette_indexes_not_truecolor() {
    let out = stdout(&["--no-config"], RUST_SAMPLE);
    // Default syntax accents are AnsiValue (keyword=117, function=153, ...), which
    // must render as palette indexes rather than 24-bit truecolour.
    assert!(
        out.contains("\x1b[38;5;"),
        "palette index expected for accents: {out:?}"
    );
    // No truecolour foreground inside the code body (only the pretty border uses it).
    let non_border_truecolor = out
        .split(BORDER_COLOR)
        .map(|segment| segment.matches("\x1b[38;2;").count())
        .sum::<usize>();
    assert_eq!(
        non_border_truecolor, 0,
        "no truecolour syntax accent expected (border excepted): {out:?}"
    );
}

#[test]
fn external_code_theme_still_uses_truecolor() {
    let out = stdout(&["--no-config", "--code-theme", "monokai"], RUST_SAMPLE);
    assert!(
        out.contains("\x1b[38;2;"),
        "monokai must keep truecolour accents: {out:?}"
    );
}
