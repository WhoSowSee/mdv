use super::*;

#[test]
fn test_render_html_ordered_list_attributes() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"<ol start="5">
  <li>Five</li>
  <li>Six</li>
</ol>
<ol start="3" reversed>
  <li>Three</li>
  <li>Two</li>
  <li>One</li>
</ol>
<ol type="a">
  <li>Alpha</li>
  <li>Beta</li>
</ol>
<ol type="A" start="27">
  <li>Upper alpha</li>
</ol>
<ol type="i" start="4">
  <li>Lower roman</li>
</ol>
<ol type="I" start="9">
  <li>Upper roman</li>
</ol>
<ol>
  <li value="4">Explicit value</li>
  <li>After explicit value</li>
</ol>
"#,
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-E")
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let clean = strip_ansi(&String::from_utf8(output.stdout).unwrap());
    for expected in [
        "5. Five",
        "6. Six",
        "3. Three",
        "2. Two",
        "1. One",
        "a. Alpha",
        "b. Beta",
        "AA. Upper alpha",
        "iv. Lower roman",
        "IX. Upper roman",
        "4. Explicit value",
        "5. After explicit value",
    ] {
        assert!(
            clean.contains(expected),
            "missing {expected}; stdout:\n{}",
            clean
        );
    }
}

#[test]
fn test_render_html_unordered_list_type_markers() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"<ul type="circle"><li>Circle</li></ul>
<ul type="square"><li>Square</li></ul>
<ul type="disc"><li>Disc</li></ul>
<ul><li>Default</li></ul>
"#,
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-E")
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let clean = strip_ansi(&String::from_utf8(output.stdout).unwrap());
    assert!(clean.contains("◦ Circle"), "stdout:\n{}", clean);
    assert!(clean.contains("▪ Square"), "stdout:\n{}", clean);
    assert!(clean.contains("• Disc"), "stdout:\n{}", clean);
    assert!(clean.contains("- Default"), "stdout:\n{}", clean);
}

#[test]
fn test_render_html_option_formats_html_tables() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"<table>
  <thead>
    <tr>
      <th align="left">Project</th>
      <th style="text-align:center">Status</th>
      <th align="right">Asset</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><strong>Alpha</strong></td>
      <td><a href="https://example.com/ready">Ready</a></td>
      <td><img src="logo.gif" alt="Logo"></td>
    </tr>
    <tr>
      <td>Beta</td>
      <td>Blocked</td>
      <td><video src="demo.mp4" title="Demo"></video></td>
    </tr>
  </tbody>
</table>
"#,
    )
    .unwrap();

    let output = mdv_cmd()
        .arg("--no-colors")
        .arg("-c")
        .arg("80")
        .arg("--render-html")
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Project"), "stdout:\n{}", stdout);
    assert!(stdout.contains("Status"), "stdout:\n{}", stdout);
    assert!(stdout.contains("Alpha"), "stdout:\n{}", stdout);
    assert!(stdout.contains("Ready"), "stdout:\n{}", stdout);
    assert!(stdout.contains("[GIF] Logo"), "stdout:\n{}", stdout);
    assert!(stdout.contains("[VIDEO] Demo"), "stdout:\n{}", stdout);
    let header_line = stdout
        .lines()
        .find(|line| line.contains("Project") && line.contains("Status") && line.contains("Asset"))
        .expect("rendered HTML table header");
    let alpha_line = stdout
        .lines()
        .find(|line| line.contains("Alpha") && line.contains("Ready"))
        .expect("rendered HTML table row");
    assert_eq!(header_line.matches('│').count(), 2, "stdout:\n{}", stdout);
    assert_eq!(alpha_line.matches('│').count(), 2, "stdout:\n{}", stdout);
    assert!(!stdout.contains("<table"), "stdout:\n{}", stdout);
    assert!(!stdout.contains("<td"), "stdout:\n{}", stdout);
}
