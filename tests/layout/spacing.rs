use super::*;

fn render_with_block_spacing(markdown: &str, spacing: &str, extra_args: &[&str]) -> String {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, markdown).unwrap();

    let output = mdv_cmd()
        .arg("--no-config")
        .arg("--no-colors")
        .args(extra_args)
        .arg("--block-spacing")
        .arg(spacing)
        .arg(temp_file.path())
        .output()
        .expect("mdv runs with block spacing");
    assert!(
        output.status.success(),
        "mdv execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("stdout utf8")
}

#[path = "spacing/backslash_basic.rs"]
mod backslash_basic;
#[path = "spacing/backslash_blocks.rs"]
mod backslash_blocks;
#[path = "spacing/block_spacing.rs"]
mod block_spacing;
#[path = "spacing/inline_html.rs"]
mod inline_html;
#[path = "spacing/paragraphs.rs"]
mod paragraphs;
