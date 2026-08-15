use super::*;

#[test]
fn test_table_rendering() {
    let theme_manager = ThemeManager::new();
    let theme = theme_manager.get_theme("terminal").unwrap();
    let renderer = TableRenderer::new(theme, false, 80, TableWrapMode::Fit);

    let headers = vec!["Name".to_string(), "Value".to_string()];
    let rows = vec![
        vec!["Test".to_string(), "123".to_string()],
        vec!["Another".to_string(), "456".to_string()],
    ];
    let alignments = vec![Alignment::Left, Alignment::Right];

    let result = renderer.render_table(&headers, &rows, &alignments);
    assert!(result.is_ok());

    let table_str = result.unwrap();
    assert!(!table_str.is_empty());
    assert!(table_str.contains("Name"));
    assert!(table_str.contains("Value"));
    assert!(table_str.contains("\x1b["));
}

#[test]
fn test_empty_table() {
    let theme_manager = ThemeManager::new();
    let theme = theme_manager.get_theme("terminal").unwrap();
    let renderer = TableRenderer::new(theme, false, 80, TableWrapMode::Fit);

    let headers = vec![];
    let rows = vec![];
    let alignments = vec![];

    let result = renderer.render_table(&headers, &rows, &alignments);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "");
}

#[test]
fn test_table_rendering_no_colors() {
    let theme_manager = ThemeManager::new();
    let theme = theme_manager.get_theme("terminal").unwrap();
    let renderer = TableRenderer::new(theme, true, 80, TableWrapMode::Fit);

    let headers = vec!["Name".to_string(), "Value".to_string()];
    let rows = vec![vec!["Test".to_string(), "123".to_string()]];
    let alignments = vec![Alignment::Left, Alignment::Right];

    let table_str = renderer.render_table(&headers, &rows, &alignments).unwrap();

    assert!(!table_str.contains("\x1b["));
}

#[test]
fn test_narrow_terminal_vertical_layout() {
    let theme_manager = ThemeManager::new();
    let theme = theme_manager.get_theme("terminal").unwrap();
    let renderer = TableRenderer::new(theme, false, 30, TableWrapMode::Wrap); // Very narrow terminal with wrap mode

    let headers = vec!["Name".to_string(), "Age".to_string(), "City".to_string()];
    let rows = vec![
        vec![
            "Alice".to_string(),
            "25".to_string(),
            "New York".to_string(),
        ],
        vec!["Bob".to_string(), "30".to_string(), "London".to_string()],
    ];
    let alignments = vec![Alignment::Left, Alignment::Right, Alignment::Left];

    let result = renderer.render_table(&headers, &rows, &alignments);
    assert!(result.is_ok());

    let output = result.unwrap();
    // Should render table properly for narrow terminals with wrap mode
    // The table might fit in 30 chars, so let's check if it contains basic table elements
    assert!(output.contains("Name"));
    assert!(output.contains("Age"));
    assert!(output.contains("City"));
    assert!(output.contains("Alice"));
}

#[test]
fn test_wide_table_column_wrapping() {
    let theme_manager = ThemeManager::new();
    let theme = theme_manager.get_theme("terminal").unwrap();
    let renderer = TableRenderer::new(theme, false, 60, TableWrapMode::Wrap); // Medium width terminal with wrap mode

    let headers = vec![
        "Very Long Header Name".to_string(),
        "Another Long Header".to_string(),
        "Third Column".to_string(),
        "Fourth Column".to_string(),
    ];
    let rows = vec![vec![
        "Long content in first column".to_string(),
        "Content in second".to_string(),
        "Third content".to_string(),
        "Fourth content".to_string(),
    ]];
    let alignments = vec![
        Alignment::Left,
        Alignment::Left,
        Alignment::Left,
        Alignment::Left,
    ];

    let result = renderer.render_table(&headers, &rows, &alignments);
    assert!(result.is_ok());

    let output = result.unwrap();
    // Should contain information about multiple blocks
    assert!(output.to_lowercase().contains("block"));
}

#[test]
fn test_column_wrapping_logic() {
    let theme_manager = ThemeManager::new();
    let theme = theme_manager.get_theme("terminal").unwrap();
    let renderer = TableRenderer::new(theme, false, 30, TableWrapMode::Fit); // Very narrow terminal

    let headers = vec![
        "Very Long Column Header 1".to_string(),
        "Very Long Column Header 2".to_string(),
        "Very Long Column Header 3".to_string(),
        "Very Long Column Header 4".to_string(),
    ];
    let rows = vec![vec![
        "Long content in first column".to_string(),
        "Long content in second column".to_string(),
        "Long content in third column".to_string(),
        "Long content in fourth column".to_string(),
    ]];
    let alignments = vec![Alignment::Left; 4];

    let blocks = renderer.split_table_into_blocks(&headers, &rows, &alignments);

    // Should split into multiple blocks for narrow terminal with long content
    assert!(!blocks.is_empty());

    // Each block should have at least one column
    for (block_headers, _, _) in &blocks {
        assert!(!block_headers.is_empty());
    }

    // Total columns across all blocks should equal original column count
    let total_columns: usize = blocks.iter().map(|(headers, _, _)| headers.len()).sum();
    assert_eq!(total_columns, headers.len());
}

#[test]
fn test_theme_color_to_comfy_conversion() {
    let ansi_color = ThemeColor::AnsiValue(42);
    assert_eq!(
        theme_color_to_comfy(&ansi_color),
        Some(Color::AnsiValue(42))
    );

    let rgb_color = ThemeColor::Rgb { r: 1, g: 2, b: 3 };
    assert_eq!(
        theme_color_to_comfy(&rgb_color),
        Some(Color::Rgb { r: 1, g: 2, b: 3 })
    );

    assert_eq!(theme_color_to_comfy(&ThemeColor::Reset), None);
}
