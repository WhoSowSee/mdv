use super::*;

#[test]
fn footnotes_with_duplicate_names_follow_reference_order() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "First[^a] second[^b] third[^a].\n\n[^a]: Alpha\n[^b]: Beta\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-colors")
        .arg("--cols")
        .arg("80")
        .arg(temp_file.path());

    let output = cmd.output().expect("run mdv with duplicate footnotes");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    let first_alpha = stdout.find("[^a] Alpha").expect("first alpha");
    let beta = stdout.find("[^b] Beta").expect("beta");
    let second_alpha = stdout[first_alpha + 1..]
        .find("[^a] Alpha")
        .map(|offset| offset + first_alpha + 1)
        .expect("second alpha");

    assert!(
        first_alpha < beta && beta < second_alpha,
        "expected duplicate footnotes in reference order, stdout: {}",
        stdout
    );
}

#[test]
fn footnotes_with_duplicate_names_preserve_bodies() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "One[^clear] Two[^clear]\n\n[^clear]: clear\n[^clear]: updated\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-colors")
        .arg("--cols")
        .arg("120")
        .arg(temp_file.path());

    let output = cmd
        .output()
        .expect("run mdv with duplicate footnote definitions");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    let first = stdout
        .find("[^clear] clear")
        .expect("first clear footnote present");
    let second = stdout[first + 1..]
        .find("[^clear] updated")
        .map(|offset| offset + first + 1)
        .expect("second clear footnote present");
    assert!(
        first < second,
        "footnote bodies must follow definition order: {}",
        stdout
    );
}

#[test]
fn footnotes_render_at_document_end_with_separator() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        "Intro with note[^a].\n\nSecond line.\n\n[^a]: Footnote content\n",
    )
    .unwrap();

    let mut cmd = mdv_cmd();
    cmd.arg("--no-colors")
        .arg("--cols")
        .arg("80")
        .arg(temp_file.path());

    let output = cmd.output().expect("run mdv with footnotes");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    let separator_count = stdout
        .lines()
        .filter(|line| line.trim_start().starts_with('◇'))
        .count();
    assert_eq!(
        separator_count, 1,
        "footnote separator should appear once: {}",
        stdout
    );
    assert!(
        stdout.contains("[^a] Footnote content"),
        "footnote body should be rendered: {}",
        stdout
    );
    assert!(
        !stdout.contains("[^a]: Footnote content"),
        "raw definition must be hidden"
    );
}
