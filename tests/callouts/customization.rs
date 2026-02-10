use super::*;

#[test]
fn test_custom_callout_icon_applies() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!custom]\n> Example text\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple:show-icons")
        .arg("--custom-callout")
        .arg("custom:icon=*")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for custom callout icon");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        stdout.contains("┃ *  Custom"),
        "expected custom callout icon, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_custom_callout_color_keeps_default_icon_for_builtin() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!tip]\n> Example text\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple:show-icons")
        .arg("--custom-callout")
        .arg("tip:color=red")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for custom callout color");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        stdout.contains("┃   Tip"),
        "expected default tip icon to remain, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_label_override_keeps_type_icon_over_custom_label() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!note] custom\n> Example text\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple:show-icons")
        .arg("--custom-callout")
        .arg("custom:icon=*")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout label override");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        stdout.contains("┃   custom"),
        "expected type icon to remain for label override, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_label_override_requires_space() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!info]Myname\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout label spacing");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        stdout.contains("┃ [Info]"),
        "expected default callout label, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("[Myname]"),
        "expected no custom label without space, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("Myname"),
        "expected inline text to be ignored, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_inline_label_without_space_does_not_add_extra_blank_line() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        ">[!info]Информация\n>dsadasasasasasasasasasasasasasasasasasasasasas\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for inline callout label spacing");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let lines: Vec<&str> = stdout.lines().collect();

    let header_idx = lines
        .iter()
        .position(|line| *line == "┃ [Info]")
        .expect("callout header present");
    let spacer_line = lines.get(header_idx + 1).copied().unwrap_or_default();
    let body_line = lines.get(header_idx + 2).copied().unwrap_or_default();

    assert_eq!(
        spacer_line, "┃ ",
        "expected single spacer line after header, stdout:\n{}",
        stdout
    );
    assert!(
        body_line.contains("dsadasa"),
        "expected body to follow spacer line, stdout:\n{}",
        stdout
    );
    let extra_spacers = lines
        .iter()
        .skip(header_idx + 2)
        .take_while(|line| **line == "┃ ")
        .count();
    assert_eq!(
        extra_spacers, 0,
        "expected no extra spacer lines, stdout:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("Информация"),
        "expected inline label to be ignored, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_fold_icons_show_when_enabled() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "> [!info]+\n> Expanded\n\n> [!info]-\n> Collapsed\n",
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple:show-icons;fold-icons")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout fold icons");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        stdout.contains("Info "),
        "expected expanded fold icon, stdout:\n{}",
        stdout
    );
    assert!(
        stdout.contains("Info "),
        "expected collapsed fold icon, stdout:\n{}",
        stdout
    );
}

#[test]
fn test_callout_fold_icons_hidden_without_show_icons() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "> [!info]+\n> Example\n").unwrap();

    let output = mdv_cmd()
        .arg("-A")
        .arg("-W")
        .arg("none")
        .arg("--callout-style")
        .arg("simple:show-icons")
        .arg(temp_file.path())
        .output()
        .expect("mdv runs for callout fold icon visibility");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(
        !stdout.contains("") && !stdout.contains(""),
        "expected no fold icons without show-icons, stdout:\n{}",
        stdout
    );
}
