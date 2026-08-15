use super::*;
use mdv::utils::{display_width, strip_ansi};

fn render_link_wrapping_case(
    markdown: &str,
    width: &str,
    wrap_mode: &str,
    link_style: &str,
) -> String {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, markdown).unwrap();

    let mut cmd = mdv_cmd();
    let output = cmd
        .env("MDV_NO_COLOR", "false")
        .args([
            "--no-config",
            "--cols",
            width,
            "--wrap",
            wrap_mode,
            "--link-style",
            link_style,
        ])
        .arg(temp_file.path())
        .output()
        .expect("render link wrapping case");

    assert!(
        output.status.success(),
        "mdv execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("stdout is valid utf-8")
}

fn render_basic_table(configure: impl FnOnce(&mut Command)) -> String {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "| Col1 | Col2 |\n|------|------|\n| A    | B    |\n| C    | D    |",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    configure(&mut cmd);
    let output = cmd
        .arg(temp_file.path())
        .output()
        .expect("render basic table");

    assert!(
        output.status.success(),
        "mdv execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("stdout is valid utf-8")
}

#[path = "basic/links.rs"]
mod links;
#[path = "basic/tables.rs"]
mod tables;
