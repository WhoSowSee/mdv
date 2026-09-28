use super::*;

fn render_details(input: &str, width: &str, extra_args: &[&str]) -> String {
    let file = NamedTempFile::new().unwrap();
    fs::write(&file, input).unwrap();
    let output = mdv_cmd()
        .args(["--color", "never", "-E", "-c", width])
        .args(extra_args)
        .arg(file.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn details_keep_readme_images_in_a_compact_tree_across_blank_lines() {
    let output = render_details(
        r#"<details>
  <summary><code>terminal</code></summary>

  <div style="text-align: center;">
    <img src="one.webp" alt="First preview">
  </div>
  <div style="text-align: center;">
    <img src="two.webp" alt="Second preview">
  </div>
</details>

<details open><summary>monokai</summary><p>Body</p></details>
<div align="right">Outside</div>
"#,
        "50",
        &[],
    );
    let lines: Vec<_> = output.lines().map(str::trim_end).collect();
    let start = lines
        .iter()
        .position(|line| *line == " `terminal`")
        .unwrap();
    assert_eq!(
        &lines[start..start + 5],
        [
            " `terminal`",
            "│               [IMAGE] First preview",
            "│              [IMAGE] Second preview",
            " monokai",
            "│  Body"
        ],
        "{output}"
    );
    assert!(
        lines.contains(&format!("{}Outside", " ".repeat(43)).as_str()),
        "{output}"
    );
    assert!(!output.contains('\x1b'), "{output}");
}

#[test]
fn details_preserve_nested_content_source_lines_and_block_spacing() {
    let output = render_details(
        r#"<details><summary>Outer</summary>

Markdown **body** and `code`.

<details><summary>Inner</summary>

<p>Nested body</p>
</details>
<p>Outer tail</p>

<pre>first

    last</pre>
<ul>
  <li>First item</li>
  <li>Last item</li>
</ul>
</details>

After
"#,
        "60",
        &["--line-numbers", "source"],
    );
    assert!(output.contains("16 │  - Last item"), "{output}");
    let output = output
        .lines()
        .map(|line| line.chars().skip(3).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(output.contains("│  Markdown body and `code`."), "{output}");
    assert!(output.contains("│   Inner"), "{output}");
    assert!(output.contains("│  │  Nested body"), "{output}");
    assert!(output.contains("│  Outer tail"), "{output}");
    assert!(output.contains("│  first\n│  \n│      last"), "{output}");
    assert!(
        output.contains("│  - First item\n│  - Last item\n\nAfter"),
        "{output}"
    );

    let body =
        "<div>Before</div><p>Paragraph</p><ul><li>Item</li></ul><pre>Code</pre><div>After</div>";
    for (spacing, gap) in [
        (
            "paragraph:top=0,bottom=0;unordered-list:top=0,bottom=0;code-block:top=0,bottom=0",
            "\n",
        ),
        (
            "paragraph:top=2,bottom=3;unordered-list:top=1,bottom=2;code-block:top=2,bottom=1",
            "\n\n\n\n",
        ),
    ] {
        let args = ["--block-spacing", spacing];
        let outside = render_details(body, "60", &args);
        let inside = render_details(
            &format!("<details><summary>Title</summary>{body}</details>"),
            "60",
            &args,
        );
        let inside = inside
            .lines()
            .skip(1)
            .map(|line| line.strip_prefix("│  ").unwrap_or(line).trim_end())
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(inside, outside.trim_end(), "spacing: {spacing}");
        assert!(
            outside.contains(&format!("Paragraph{gap}- Item")),
            "{outside}"
        );
    }
}
