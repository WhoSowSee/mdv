use super::*;
use crate::markdown::MarkdownProcessor;

#[test]
fn renderer_exposes_pager_status_bar_transparency() {
    let config = Config {
        custom_theme: Some("pager_status_bar_transparent=true".to_string()),
        ..Config::default()
    };

    let renderer = TerminalRenderer::new(&config).unwrap();

    assert!(renderer.pager_status_bar_transparent());
}

fn contains_numbered_line(output: &str, number: usize) -> bool {
    let prefix = format!("{number} ");
    output
        .lines()
        .any(|line| line.trim_start().starts_with(&prefix))
}

#[test]
fn pager_render_builds_source_navigation_for_each_numbering_mode() {
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
            no_colors: true,
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
        let rendered = TerminalRenderer::new(&config)
            .unwrap()
            .render_document_for_pager(document)
            .unwrap();

        assert!(rendered.source_lines.contains(&Some(5)));
        assert_eq!(rendered.source_lines.len(), rendered.output.lines().count());
        assert!(!rendered.output.contains('\u{2063}'));
        let target = line_numbers.map(|options| options.target);
        let source_output = match &rendered.source_view {
            SourceNumberedView::Current => {
                assert_eq!(target, Some(LineNumberTarget::Source));
                &rendered.output
            }
            SourceNumberedView::Alternate {
                output,
                source_lines,
            } => {
                assert_ne!(target, Some(LineNumberTarget::Source));
                assert!(source_lines.contains(&Some(5)));
                assert_eq!(source_lines.len(), output.lines().count());
                output
            }
        };
        assert!(!source_output.contains('\u{2063}'));
        assert!(contains_numbered_line(source_output, 5));

        match target {
            Some(LineNumberTarget::Source) => {}
            Some(LineNumberTarget::Rendered) => {
                assert!(contains_numbered_line(&rendered.output, 4));
                assert!(!contains_numbered_line(&rendered.output, 5));
            }
            None => {
                assert!(!rendered.output.lines().any(|line| {
                    line.trim_start()
                        .chars()
                        .next()
                        .is_some_and(|character| character.is_ascii_digit())
                }));
            }
        }
    }
}
