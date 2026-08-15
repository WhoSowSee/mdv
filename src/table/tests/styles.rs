use super::*;

#[test]
fn test_table_link_text_keeps_default_color() {
    let theme_manager = ThemeManager::new();
    let theme = theme_manager.get_theme("terminal").unwrap();
    let renderer = TableRenderer::new(theme, false, 80, TableWrapMode::Fit);

    let link_text = "Link text";
    let formatted_link_text = format!("\x1b[4m{}\x1b[24m", link_text);
    let styled_reference = create_style(theme, ThemeElement::Link).apply("[1]", false);

    let headers = vec!["Col".to_string()];
    let rows = vec![vec![format!("{}{}", formatted_link_text, styled_reference)]];
    let alignments = vec![Alignment::Left];

    let table_output = renderer
        .render_table(&headers, &rows, &alignments)
        .expect("table rendered");

    let data_line = table_output
        .lines()
        .find(|line| line.contains(link_text))
        .expect("data row present");
    assert!(data_line.contains(&styled_reference));
    let stripped_line = crate::utils::strip_ansi(data_line);
    assert!(stripped_line.contains("Link text[1]"));

    let prefix_len = styled_reference
        .find("[1]")
        .expect("styled reference contains '[1]'");
    let color_prefix = &styled_reference[..prefix_len];

    let reference_pos = data_line
        .find(&styled_reference)
        .expect("styled reference present");
    let before_reference = &data_line[..reference_pos];

    assert!(data_line.contains(color_prefix));
    assert!(
        !before_reference.contains(color_prefix),
        "link color prefix should not tint link text; line={:?}",
        data_line
    );
}

#[test]
fn test_table_mixed_inline_code_keeps_plain_text_unstyled() {
    let theme_manager = ThemeManager::new();
    let theme = theme_manager.get_theme("terminal").unwrap();
    let renderer = TableRenderer::new(theme, false, 100, TableWrapMode::Fit);

    let plain_text = "Versioned little-endian emulator snapshot with magic ";
    let styled_code = create_style(theme, ThemeElement::Code).apply("`K580`", false);
    let rows = vec![vec![format!("{plain_text}{styled_code}.")]];
    let output = renderer
        .render_table(&["Purpose".to_string()], &rows, &[Alignment::Left])
        .expect("table rendered");
    let data_line = output
        .lines()
        .find(|line| strip_ansi(line).contains(plain_text))
        .expect("data row present");
    let code_color_prefix = styled_code
        .split('`')
        .next()
        .expect("styled code has a visible delimiter");
    let plain_text_end = data_line.find(plain_text).expect("plain text is present");
    assert!(
        !data_line[..plain_text_end].contains(code_color_prefix),
        "inline code color must not start before plain text; line={data_line:?}"
    );
    assert!(
        data_line[plain_text_end..].contains(code_color_prefix),
        "inline code color must be preserved for the code span; line={data_line:?}"
    );
}

#[test]
fn test_table_mixed_attributes_remain_scoped() {
    let theme_manager = ThemeManager::new();
    let theme = theme_manager.get_theme("terminal").unwrap();
    let renderer = TableRenderer::new(theme, false, 100, TableWrapMode::Fit);

    let strong = create_style(theme, ThemeElement::Strong).apply("strong", false);
    let emphasis = create_style(theme, ThemeElement::Emphasis).apply("emphasis", false);
    let content = format!("plain {strong} middle {emphasis} tail");
    let output = renderer
        .render_table(
            &["Content".to_string()],
            &[vec![content.clone()]],
            &[Alignment::Left],
        )
        .expect("table rendered");
    let data_line = output
        .lines()
        .find(|line| strip_ansi(line).contains("plain strong middle emphasis tail"))
        .expect("mixed-format data row present");

    assert!(
        data_line.contains(&content),
        "cell should preserve each ANSI span without widening its scope: {data_line:?}"
    );
}

#[test]
fn test_table_inline_link_preserves_text_color() {
    let theme_manager = ThemeManager::new();
    let theme = theme_manager.get_theme("terminal").unwrap();
    let renderer = TableRenderer::new(theme, false, 80, TableWrapMode::Fit);

    let link_text = "Link text";
    let formatted_link_text = format!("\x1b[4m{}\x1b[24m", link_text);
    let url_part = "(https://example.com)".to_string();
    let styled_url = create_style(theme, ThemeElement::Link).apply(&url_part, false);

    let headers = vec!["Col".to_string()];
    let rows = vec![vec![format!("{}{}", formatted_link_text, styled_url)]];
    let alignments = vec![Alignment::Left];

    let table_output = renderer
        .render_table(&headers, &rows, &alignments)
        .expect("table rendered");

    let data_line = table_output
        .lines()
        .find(|line| line.contains(link_text))
        .expect("data row present");

    assert!(data_line.contains(&styled_url));

    let stripped_line = crate::utils::strip_ansi(data_line);
    assert!(stripped_line.contains(&format!("{}{}", link_text, url_part)));

    let prefix_len = styled_url
        .find(&url_part)
        .expect("styled url contains raw url");
    let color_prefix = &styled_url[..prefix_len];

    let reference_pos = data_line.find(&styled_url).expect("styled url present");
    let before_reference = &data_line[..reference_pos];

    assert!(data_line.contains(color_prefix));
    assert!(
        !before_reference.contains(color_prefix),
        "link color prefix should not tint link text; line={:?}",
        data_line
    );
}
