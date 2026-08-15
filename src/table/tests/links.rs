use super::*;

#[test]
fn test_table_inline_wrapped_url_keeps_link_color() {
    let theme_manager = ThemeManager::new();
    let theme = theme_manager.get_theme("terminal").unwrap();
    let renderer = TableRenderer::new(theme, false, 36, TableWrapMode::Fit);

    let link_text = "dash";
    let formatted_link_text = format!("\x1b[4m{}\x1b[24m", link_text);
    let url_part = "(https://example.com/dashboard/alpha)".to_string();
    let styled_url = create_style(theme, ThemeElement::Link).apply(&url_part, false);

    let headers = vec!["Link".to_string()];
    let rows = vec![vec![format!("{}{}", formatted_link_text, styled_url)]];
    let alignments = vec![Alignment::Left];

    let table_output = renderer
        .render_table(&headers, &rows, &alignments)
        .expect("table rendered");
    let stripped = crate::utils::strip_ansi(&table_output);

    assert!(
        !stripped.contains(&url_part),
        "url should be wrapped in narrow table, got:\n{}",
        stripped
    );

    let prefix_len = styled_url
        .find(&url_part)
        .expect("styled url contains raw url");
    let color_prefix = &styled_url[..prefix_len];

    assert!(
        table_output.matches(color_prefix).count() >= 2,
        "wrapped url should keep link color on every fragment, output:\n{:?}",
        table_output
    );
}

#[test]
fn test_fragmented_clickable_link_prefers_nearest_cell_match() {
    let guide_text = "Guide documentation".to_string();
    let api_text = "API documentation".to_string();
    let guide_url = "https://example.com/docs/guide";
    let api_url = "https://example.com/docs/api";
    let raw_table = concat!(
        "│ Guide docum ┆ API documen │\n",
        "│ entation    ┆ tation      │"
    )
    .to_string();
    let replacements = vec![
        (
            guide_text.clone(),
            format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", guide_url, guide_text),
        ),
        (
            api_text.clone(),
            format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", api_url, api_text),
        ),
    ];

    let styled_table = apply_clickable_link_replacements(raw_table, &replacements);
    let lines: Vec<&str> = styled_table.lines().collect();
    assert_eq!(lines.len(), 2, "expected two-line wrapped row");

    let guide_open = format!("\x1b]8;;{}\x1b\\", guide_url);
    let api_open = format!("\x1b]8;;{}\x1b\\", api_url);
    assert!(
        lines
            .iter()
            .all(|line| line.matches(&guide_open).count() == 1),
        "each guide fragment should keep its own OSC 8 target: {styled_table:?}"
    );
    assert!(
        lines
            .iter()
            .all(|line| line.matches(&api_open).count() == 1),
        "each API fragment should keep its own OSC 8 target: {styled_table:?}"
    );
}

#[test]
fn test_styled_wrapper_prefers_visible_text_for_osc8_links() {
    let plain = "docs";
    let styled = format!(
        "\x1b]8;;https://example.com/{}/path\x1b\\{}\x1b]8;;\x1b\\",
        plain, plain
    );

    let (prefix, suffix) = styled_wrapper(&styled, plain).expect("wrapper parsed");
    assert!(
        prefix.ends_with("\x1b\\"),
        "expected wrapper to target visible text segment, prefix={:?}",
        prefix
    );
    assert_eq!(format!("{}{}{}", prefix, plain, suffix), styled);
}
