use super::*;
use crate::markdown::MarkdownProcessor;

#[test]
fn renderer_exposes_pager_palette() {
    let config = Config {
        custom_theme: Some("pager:transparent=true;pager:title=red;pager:selection:background=darkblue;pager:search_current:text=reset".to_string()),
        ..Config::default()
    };

    let renderer = TerminalRenderer::new(&config, OutputStyle::Disabled).unwrap();

    assert!(renderer.pager_theme().transparent);
    assert_eq!(renderer.pager_theme().title, Some(crate::theme::Color::Red));
    assert_eq!(
        renderer.pager_theme().selection.background,
        Some(crate::theme::Color::DarkBlue)
    );
    assert_eq!(
        renderer.pager_theme().search_current.text,
        Some(crate::theme::Color::Reset)
    );
}

fn contains_numbered_line(output: &str, number: usize) -> bool {
    let prefix = format!("{number} ");
    output
        .lines()
        .any(|line| line.trim_start().starts_with(&prefix))
}

#[test]
fn pager_render_preserves_heading_indent_on_paragraph_source_lines() {
    use crate::cli::TextWrapMode;

    for (markdown, starts) in [
        (
            "## Heading\n\nFirst line\nSecond line\n",
            ["First line", "Second line"].as_slice(),
        ),
        (
            include_str!("../../../docs/examples/sample.md"),
            [
                "Regular text",
                "GitHub Flavored Markdown",
                "Inline code such as",
            ]
            .as_slice(),
        ),
    ] {
        for (cols, wrap, output_style) in [
            (30, TextWrapMode::Char, OutputStyle::Enabled),
            (80, TextWrapMode::Word, OutputStyle::Disabled),
            (80, TextWrapMode::None, OutputStyle::Disabled),
        ] {
            let config = Config {
                cols: Some(cols),
                cols_from_cli: true,
                wrap,
                ..Config::default()
            };
            let mut processor_config = config.clone();
            processor_config.line_numbers = Some(LineNumberOptions {
                target: LineNumberTarget::Source,
                separator: false,
            });
            let document = MarkdownProcessor::new(&processor_config)
                .parse_document(markdown)
                .unwrap();
            let rendered = TerminalRenderer::new(&config, output_style)
                .unwrap()
                .render_document_for_pager(document)
                .unwrap();

            for target in [
                None,
                Some(LineNumberTarget::Rendered),
                Some(LineNumberTarget::Source),
            ] {
                let view = (rendered.render_view)(target).unwrap();
                let output = crate::utils::strip_ansi(&view.output);
                let lines: Vec<_> = output.lines().collect();
                assert_eq!(view.source_lines.len(), lines.len());
                for text in starts {
                    let source_line = markdown
                        .lines()
                        .position(|line| line.starts_with(text))
                        .unwrap()
                        + 1;
                    let row = view
                        .source_lines
                        .iter()
                        .position(|line| *line == Some(source_line))
                        .unwrap();
                    let line = if target.is_some() {
                        lines[row].trim_start().split_once(' ').unwrap().1
                    } else {
                        lines[row]
                    };
                    assert!(line.trim_start().starts_with(text), "{line:?}");
                    assert_eq!(line.len() - line.trim_start().len(), 2, "{line:?}");
                    if !matches!(wrap, TextWrapMode::None) {
                        assert!(crate::utils::display_width(lines[row]) <= cols, "{line:?}");
                    }
                }
            }
        }
    }
}

#[test]
fn pager_render_builds_all_line_number_modes_from_each_starting_mode() {
    let modes = [
        None,
        Some(LineNumberOptions::default()),
        Some(LineNumberOptions {
            target: LineNumberTarget::Source,
            separator: true,
        }),
    ];

    for line_numbers in modes {
        let config = Config {
            cols: Some(80),
            line_numbers,
            ..Config::default()
        };
        let mut processor_config = config.clone();
        processor_config.line_numbers = Some(LineNumberOptions {
            target: LineNumberTarget::Source,
            separator: false,
        });
        let document = MarkdownProcessor::new(&processor_config)
            .parse_document("# Title\n\n> First\n>\n> Second")
            .unwrap();
        let pager_render = TerminalRenderer::new(&config, OutputStyle::Disabled)
            .unwrap()
            .render_document_for_pager(document)
            .unwrap();

        let target = line_numbers.map(|options| options.target);
        assert_eq!(pager_render.initial_target, target);
        let unnumbered = (pager_render.render_view)(None).unwrap();
        let rendered_view = (pager_render.render_view)(Some(LineNumberTarget::Rendered)).unwrap();
        let source = (pager_render.render_view)(Some(LineNumberTarget::Source)).unwrap();

        for view in [&unnumbered, &rendered_view, &source] {
            assert!(view.source_lines.contains(&Some(5)));
            assert_eq!(view.source_lines.len(), view.output.lines().count());
            assert!(!view.output.contains('\u{2063}'));
        }

        assert!(!unnumbered.output.lines().any(|line| {
            line.trim_start()
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_digit())
        }));
        assert!(contains_numbered_line(&rendered_view.output, 4));
        assert!(!contains_numbered_line(&rendered_view.output, 5));
        assert!(contains_numbered_line(&source.output, 5));

        let separator = line_numbers.is_some_and(|options| options.separator);
        assert_eq!(rendered_view.output.starts_with("1 │ "), separator);
        assert_eq!(source.output.starts_with("1 │ "), separator);
    }
}

#[test]
fn pager_render_preserves_callout_layout_with_source_metadata() {
    for style in [
        crate::cli::CalloutStyle::Pretty,
        crate::cli::CalloutStyle::Simple,
    ] {
        let mut config = Config {
            cols: Some(80),
            ..Config::default()
        };
        config.callout_style.style = style;
        let mut processor_config = config.clone();
        processor_config.line_numbers = Some(LineNumberOptions {
            target: LineNumberTarget::Source,
            separator: false,
        });
        let document = MarkdownProcessor::new(&processor_config)
            .parse_document("> [!IMPORTANT]\n>\n> Body\n")
            .unwrap();
        let pager_render = TerminalRenderer::new(&config, OutputStyle::Disabled)
            .unwrap()
            .render_document_for_pager(document)
            .unwrap();
        let (header, body, marker_count) = match style {
            crate::cli::CalloutStyle::Pretty => ("╭─ Important ", "│ Body", 0),
            crate::cli::CalloutStyle::Simple => ("┃ [Important]", "┃ Body", 1),
        };
        let unnumbered = (pager_render.render_view)(None).unwrap();
        let rendered_view = (pager_render.render_view)(Some(LineNumberTarget::Rendered)).unwrap();
        let source = (pager_render.render_view)(Some(LineNumberTarget::Source)).unwrap();

        for view in [&unnumbered, &rendered_view, &source] {
            assert_eq!(view.output.lines().count(), 3, "{}", view.output);
            assert_eq!(view.output.matches("[Important]").count(), marker_count);
            assert!(view.output.contains(body), "{}", view.output);
        }
        assert!(unnumbered.output.starts_with(header));
        assert!(source.source_lines.contains(&Some(1)));
        assert!(source.source_lines.contains(&Some(3)));
    }
}

#[test]
fn pager_render_preserves_display_math_geometry_and_source_line() {
    let mut config = Config {
        cols: Some(40),
        ..Config::default()
    };
    config.code_block_style.style = crate::cli::CodeBlockStyle::Simple;
    config.math_block_style = crate::cli::MathBlockStyle::Simple;
    let mut processor_config = config.clone();
    processor_config.line_numbers = Some(LineNumberOptions {
        target: LineNumberTarget::Source,
        separator: false,
    });
    let document = MarkdownProcessor::new(&processor_config)
        .with_extended_math(true)
        .parse_document("\\[\n\\frac{a+b}{c+d}\n\\]\n")
        .unwrap();
    let rendered = TerminalRenderer::new(&config, OutputStyle::Disabled)
        .unwrap()
        .render_document_for_pager(document)
        .unwrap();

    let unnumbered = (rendered.render_view)(None).unwrap();
    let rendered_view = (rendered.render_view)(Some(LineNumberTarget::Rendered)).unwrap();
    let source = (rendered.render_view)(Some(LineNumberTarget::Source)).unwrap();
    for view in [&unnumbered, &rendered_view, &source] {
        assert!(view.output.contains("a+b") && view.output.contains("c+d"));
        assert!(view.output.contains("───"), "{}", view.output);
        assert!(!view.output.contains('\u{2062}'));
        assert_eq!(view.source_lines.len(), view.output.lines().count());
    }
    assert_eq!(
        source
            .source_lines
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>(),
        [1]
    );
}
