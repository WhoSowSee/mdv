use mdv::utils::strip_ansi;
use std::{fs, process::Command};
use tempfile::NamedTempFile;

fn render_output(markdown: &str, extra_args: &[&str], no_colors: bool) -> String {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, markdown).unwrap();

    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("mdv"));
    command.args(["--no-config", "-c", "120", "--render-html"]);
    if no_colors {
        command.arg("-A");
    }
    let output = command
        .args(extra_args)
        .arg(temp_file.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).unwrap()
}

fn render(markdown: &str, extra_args: &[&str]) -> String {
    strip_ansi(&render_output(markdown, extra_args, true))
}

fn line_index(lines: &[&str], needle: &str) -> usize {
    lines
        .iter()
        .position(|line| line.contains(needle))
        .unwrap_or_else(|| panic!("missing {needle:?} in:\n{}", lines.join("\n")))
}

#[test]
fn html_nested_content_keeps_multiline_table_structure() {
    let stdout = render(
        r#"Before

| Content type | Nested content |
| --- | --- |
| Inner table | <table><thead><tr><th>Name</th><th>Value</th></tr></thead><tbody><tr><td>Alpha</td><td>1</td></tr><tr><td>Beta</td><td>2</td></tr></tbody></table> |
| List | <ul><li>First item</li><li>Second item<ul><li>Nested item</li></ul></li></ul> |
| Checkboxes | <ul><li><input type="checkbox" checked disabled> Completed task</li><li><input type="checkbox" disabled> Pending task</li></ul> |
| Definitions | <dl><dt>Markdown</dt><dd>A lightweight markup language.</dd><dt>mdv</dt><dd>A terminal Markdown renderer.</dd></dl> |
"#,
        &[],
    );
    let lines: Vec<_> = stdout.lines().collect();

    let before = line_index(&lines, "Before");
    let header = lines
        .iter()
        .position(|line| line.contains("Content type") && line.contains("Nested content"))
        .expect("rendered table header");
    assert_eq!(header, before + 2, "stdout:\n{stdout}");
    assert!(lines[before + 1].trim().is_empty(), "stdout:\n{stdout}");

    let inner_header = lines
        .iter()
        .position(|line| line.contains("Name") && line.contains("Value"))
        .expect("nested table header");
    let alpha = lines
        .iter()
        .position(|line| line.contains("Alpha") && line.contains('1'))
        .expect("nested table first row");
    let beta = lines
        .iter()
        .position(|line| line.contains("Beta") && line.contains('2'))
        .expect("nested table second row");
    assert!(inner_header < alpha && alpha < beta, "stdout:\n{stdout}");
    assert!(!lines[inner_header].contains("Alpha"), "stdout:\n{stdout}");

    let first = line_index(&lines, "- First item");
    let second = line_index(&lines, "- Second item");
    let nested = line_index(&lines, "- Nested item");
    assert!(first < second && second < nested, "stdout:\n{stdout}");
    assert_eq!(
        lines[nested].find("- Nested item").unwrap(),
        lines[second].find("- Second item").unwrap() + 2,
        "stdout:\n{stdout}"
    );

    let completed = line_index(&lines, "Completed task");
    let pending = line_index(&lines, "Pending task");
    assert!(completed < pending, "stdout:\n{stdout}");
    assert!(
        lines[completed].contains("[✓] Completed task"),
        "stdout:\n{stdout}"
    );
    assert!(
        lines[pending].contains("[ ] Pending task"),
        "stdout:\n{stdout}"
    );
    assert!(!lines[completed].contains("- [✓]"), "stdout:\n{stdout}");
    assert!(!lines[pending].contains("- [ ]"), "stdout:\n{stdout}");

    let term_markdown = line_index(&lines, "Markdown");
    let definition_markdown = line_index(&lines, "A lightweight markup language.");
    let term_mdv = line_index(&lines, "mdv");
    let definition_mdv = line_index(&lines, "A terminal Markdown renderer.");
    assert!(
        term_markdown < definition_markdown
            && definition_markdown < term_mdv
            && term_mdv < definition_mdv,
        "stdout:\n{stdout}"
    );
    assert_eq!(definition_markdown, term_markdown + 1, "stdout:\n{stdout}");
    assert_eq!(definition_mdv, term_mdv + 1, "stdout:\n{stdout}");
}

#[test]
fn html_structural_elements_keep_table_cell_boundaries() {
    let stdout = render(
        r#"| Type | Content |
| --- | --- |
| Heading | <h3>Cell heading</h3><p>Following paragraph</p> |
| Blocks | <div><p>Paragraph</p><section>Section</section><article>Article</article><header>Header</header><footer>Footer</footer></div> |
| Quote | <blockquote><p>First quote</p><p>Second quote</p></blockquote> |
| Details | <details open><summary>Summary label</summary><p>Details body</p></details> |
| Figure | <figure><img src="chart.png" alt="Chart"><figcaption>Figure caption</figcaption></figure> |
| Rule | <div>Before rule<hr>After rule</div> |
| Hidden | visible<script>bad()</script><style>.bad{}</style><template>hidden-template</template><noscript>hidden-noscript</noscript><title>hidden-title</title> |
| Inputs | <input type="text" value="Alice"> <input type="radio" checked> <input type="radio"> <input type="button" value="Apply"> |
| Button | <button><strong>Save</strong></button> |
| Select | <select><option>First</option><option selected>Second</option></select> |
| Multiple | <select multiple><option selected>One</option><option>Two</option><option selected>Three</option></select> |
"#,
        &[],
    );
    let lines: Vec<_> = stdout.lines().collect();

    let heading = line_index(&lines, "Cell heading");
    let following = line_index(&lines, "Following paragraph");
    assert!(heading < following, "stdout:\n{stdout}");

    let block_lines = ["Paragraph", "Section", "Article", "Header", "Footer"]
        .map(|text| line_index(&lines, text));
    assert!(
        block_lines.windows(2).all(|pair| pair[0] < pair[1]),
        "stdout:\n{stdout}"
    );
    assert!(
        !lines.iter().any(|line| line.trim() == "│"),
        "stdout:\n{stdout}"
    );

    let first_quote = line_index(&lines, "First quote");
    let second_quote = line_index(&lines, "Second quote");
    assert!(first_quote < second_quote, "stdout:\n{stdout}");
    assert!(
        lines[first_quote].contains("│ First quote"),
        "stdout:\n{stdout}"
    );
    assert!(
        lines[second_quote].contains("│ Second quote"),
        "stdout:\n{stdout}"
    );

    let summary = line_index(&lines, "Summary label");
    let body = line_index(&lines, "Details body");
    let image = line_index(&lines, "[IMAGE] Chart");
    let caption = line_index(&lines, "Figure caption");
    assert!(summary < body, "stdout:\n{stdout}");
    assert!(image < caption, "stdout:\n{stdout}");
    assert!(!lines[caption].contains("ChartFigure"), "stdout:\n{stdout}");

    let before_rule = line_index(&lines, "Before rule");
    let after_rule = line_index(&lines, "After rule");
    assert!(!stdout.contains('◈'), "stdout:\n{stdout}");
    assert!(before_rule < after_rule, "stdout:\n{stdout}");
    assert!(
        lines[before_rule + 1..after_rule]
            .iter()
            .any(|line| line.contains("───")),
        "stdout:\n{stdout}"
    );

    assert!(stdout.contains("visible"), "stdout:\n{stdout}");
    for hidden in [
        "bad()",
        ".bad{}",
        "hidden-template",
        "hidden-noscript",
        "hidden-title",
    ] {
        assert!(!stdout.contains(hidden), "leaked {hidden:?}:\n{stdout}");
    }

    for expected in [
        "[Alice]",
        "(●)",
        "( )",
        "[Apply]",
        "[Save]",
        "[Second]",
        "[One, Three]",
    ] {
        assert!(stdout.contains(expected), "missing {expected:?}:\n{stdout}");
    }
}

#[test]
fn html_table_markers_and_headings_use_configured_styles() {
    let pretty = render(
        r#"| Type | Content |
| --- | --- |
| List | <ul><li>First item<ul><li>Nested item</li></ul></li><li><input type="checkbox" checked> Completed task</li><li><input type="checkbox"> Pending task</li></ul> |
"#,
        &[
            "--pretty-list",
            "type:unicode;size:small",
            "--pretty-checkbox",
            "square",
        ],
    );
    let first = pretty
        .lines()
        .find(|line| line.contains("First item"))
        .expect("first list item");
    let nested = pretty
        .lines()
        .find(|line| line.contains("Nested item"))
        .expect("nested list item");
    let completed = pretty
        .lines()
        .find(|line| line.contains("Completed task"))
        .expect("completed task");
    let pending = pretty
        .lines()
        .find(|line| line.contains("Pending task"))
        .expect("pending task");
    assert!(first.contains("⦁ First item"), "stdout:\n{pretty}");
    assert!(nested.contains("▪ Nested item"), "stdout:\n{pretty}");
    assert!(completed.contains('\u{F0132}'), "stdout:\n{pretty}");
    assert!(pending.contains('\u{F0131}'), "stdout:\n{pretty}");

    let styled = render_output(
        r#"| Type | Content |
| --- | --- |
| Heading | <h3>Cell heading</h3> |
| List | <ul><li>First item</li></ul> |
"#,
        &["--theme", "terminal"],
        false,
    );
    assert!(
        styled.contains("\u{1b}[93mCell heading\u{1b}[0m"),
        "stdout:\n{styled}"
    );
    assert!(
        styled.contains("\u{1b}[92m- \u{1b}[0mFirst item"),
        "stdout:\n{styled}"
    );
}

#[test]
fn html_code_in_table_cells_preserves_source_and_code_block_style() {
    let stdout = render(
        r#"| Type | Content |
| --- | --- |
| Code block | <pre><code class="language-rust">fn main() {&#10;    println!("hi");&#10;}</code></pre> |
| Textarea | <textarea>alpha "beta"&#10;gamma -- delta ...</textarea> |
| Inline code | <code>let s = "x"; a -- b ...</code> |
"#,
        &["--code-block-style", "pretty"],
    );

    assert!(stdout.contains("println!(\"hi\");"), "stdout:\n{stdout}");
    assert!(stdout.contains("alpha \"beta\""), "stdout:\n{stdout}");
    assert!(stdout.contains("gamma -- delta ..."), "stdout:\n{stdout}");
    assert!(
        stdout.contains("`let s = \"x\"; a -- b ...`"),
        "stdout:\n{stdout}"
    );
    assert!(
        !stdout.contains(['“', '”', '‘', '’', '–', '…']),
        "stdout:\n{stdout}"
    );
    assert!(stdout.matches('╭').count() >= 2, "stdout:\n{stdout}");
    assert!(stdout.matches('╰').count() >= 2, "stdout:\n{stdout}");
}
