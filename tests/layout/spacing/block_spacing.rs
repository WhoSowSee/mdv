use super::*;

#[test]
fn test_block_spacing_controls_each_primary_block_element() {
    let cases = [
        ("h1", "# H1"),
        ("h2", "## H2"),
        ("h3", "### H3"),
        ("h4", "#### H4"),
        ("h5", "##### H5"),
        ("h6", "###### H6"),
        ("code-block", "```text\nCODE\n```"),
        ("display-math", "$$ MATH $$"),
        ("table", "| H |\n| - |\n| CELL |"),
        ("horizontal-rule", "---"),
        ("unordered-list", "- UNORDERED\n- SECOND"),
        ("ordered-list", "1. ORDERED\n2. SECOND"),
        ("task-list", "- [ ] TASK\n- [x] DONE"),
        ("blockquote", "> QUOTED"),
        ("callout", "> [!NOTE]\n> CALLOUT"),
        ("definition-list", "TERM\n: DEFINITION"),
    ];

    for (element, block) in cases {
        let stdout = render_with_block_spacing(
            &format!("BEFORE\n\n{block}\n\nAFTER\n"),
            &format!("{element}:top=2,bottom=3"),
            &["--wrap", "none"],
        );
        let lines: Vec<&str> = stdout.lines().collect();
        let before_idx = lines
            .iter()
            .position(|line| line.trim() == "BEFORE")
            .unwrap_or_else(|| panic!("{element}: BEFORE missing, stdout:\n{stdout}"));
        let after_idx = lines
            .iter()
            .position(|line| line.trim() == "AFTER")
            .unwrap_or_else(|| panic!("{element}: AFTER missing, stdout:\n{stdout}"));
        let visible_lines: Vec<usize> = (before_idx + 1..after_idx)
            .filter(|idx| !lines[*idx].trim().is_empty())
            .collect();
        let first_visible = *visible_lines
            .first()
            .unwrap_or_else(|| panic!("{element}: block output missing, stdout:\n{stdout}"));
        let last_visible = *visible_lines
            .last()
            .unwrap_or_else(|| panic!("{element}: block output missing, stdout:\n{stdout}"));

        assert_eq!(
            first_visible,
            before_idx + 3,
            "{element}: expected two blank lines above, stdout:\n{stdout}"
        );
        assert_eq!(
            after_idx,
            last_visible + 4,
            "{element}: expected three blank lines below, stdout:\n{stdout}"
        );
    }
}

#[test]
fn test_block_spacing_collapses_adjacent_values_and_allows_zero() {
    let collapsed_stdout = render_with_block_spacing(
        "# FIRST\n## SECOND\n",
        "h1:bottom=3;h2:top=2",
        &["--heading-layout", "flat"],
    );
    let collapsed_lines: Vec<&str> = collapsed_stdout.lines().collect();
    let first_idx = collapsed_lines
        .iter()
        .position(|line| line.trim() == "FIRST")
        .expect("first heading present");
    let second_idx = collapsed_lines
        .iter()
        .position(|line| line.trim() == "SECOND")
        .expect("second heading present");
    assert_eq!(second_idx - first_idx - 1, 3);

    let zero_stdout = render_with_block_spacing(
        "BEFORE\n# HEADING\nAFTER\n",
        "paragraph:top=0,bottom=0;h1:top=0,bottom=0",
        &["--heading-layout", "flat"],
    );
    let zero_lines: Vec<&str> = zero_stdout.lines().collect();
    assert_eq!(
        zero_lines
            .iter()
            .map(|line| line.trim())
            .collect::<Vec<_>>(),
        ["BEFORE", "HEADING", "AFTER"]
    );
}

#[test]
fn test_block_spacing_controls_reference_and_footnote_blocks_independently() {
    let inline_stdout = render_with_block_spacing(
        "BEFORE\n\nSee [LINK](https://example.com).\n\nAFTER\n",
        "inline-references:top=2,bottom=3",
        &["--link-style", "inlinetable"],
    );
    let inline_lines: Vec<&str> = inline_stdout.lines().collect();
    let paragraph_idx = inline_lines
        .iter()
        .position(|line| line.contains("See LINK[1]."))
        .expect("linked paragraph present");
    let reference_idx = inline_lines
        .iter()
        .position(|line| line.contains("[1] https://example.com"))
        .expect("inline reference present");
    let after_idx = inline_lines
        .iter()
        .position(|line| line.trim() == "AFTER")
        .expect("following paragraph present");
    assert_eq!(reference_idx - paragraph_idx - 1, 2);
    assert_eq!(after_idx - reference_idx - 1, 3);

    let attached_stdout = render_with_block_spacing(
        "BEFORE[^a]\n\n[^a]: ATTACHED\n\nAFTER\n",
        "attached-footnotes:top=2,bottom=3",
        &["--footnote-style", "attached"],
    );
    let attached_lines: Vec<&str> = attached_stdout.lines().collect();
    let before_idx = attached_lines
        .iter()
        .position(|line| line.trim() == "BEFORE[^a]")
        .expect("footnote owner present");
    let attached_idx = attached_lines
        .iter()
        .position(|line| line.contains("[^a] ATTACHED"))
        .expect("attached footnote present");
    let first_border_idx = attached_lines[..attached_idx]
        .iter()
        .rposition(|line| line.starts_with('◇'))
        .expect("top footnote border present");
    let last_border_idx = attached_lines[attached_idx + 1..]
        .iter()
        .position(|line| line.starts_with('◇'))
        .map(|index| index + attached_idx + 1)
        .expect("bottom footnote border present");
    let after_idx = attached_lines
        .iter()
        .position(|line| line.trim() == "AFTER")
        .expect("following paragraph present");
    assert_eq!(first_border_idx - before_idx - 1, 2);
    assert_eq!(after_idx - last_border_idx - 1, 3);

    let end_stdout = render_with_block_spacing(
        "BODY [LINK](https://example.com) and note[^n].\n\n[^n]: NOTE\n",
        "end-references:top=2,bottom=3;endnotes:top=0",
        &["--link-style", "endtable", "--footnote-style", "endnotes"],
    );
    let end_lines: Vec<&str> = end_stdout.lines().collect();
    let body_idx = end_lines
        .iter()
        .position(|line| line.contains("BODY LINK[1]"))
        .expect("body present");
    let reference_idx = end_lines
        .iter()
        .position(|line| line.contains("[1] https://example.com"))
        .expect("end reference present");
    let endnote_border_idx = end_lines
        .iter()
        .position(|line| line.starts_with('◇'))
        .expect("endnote border present");
    assert_eq!(reference_idx - body_idx - 1, 2);
    assert_eq!(endnote_border_idx - reference_idx - 1, 3);
}
