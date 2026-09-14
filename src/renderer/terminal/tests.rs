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
        let pager_render = TerminalRenderer::new(&config)
            .unwrap()
            .render_document_for_pager(document)
            .unwrap();

        let target = line_numbers.map(|options| options.target);
        assert_eq!(pager_render.initial_target, target);

        for view in [
            &pager_render.unnumbered,
            &pager_render.rendered,
            &pager_render.source,
        ] {
            assert!(view.source_lines.contains(&Some(5)));
            assert_eq!(view.source_lines.len(), view.output.lines().count());
            assert!(!view.output.contains('\u{2063}'));
        }

        assert!(!pager_render.unnumbered.output.lines().any(|line| {
            line.trim_start()
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_digit())
        }));
        assert!(contains_numbered_line(&pager_render.rendered.output, 4));
        assert!(!contains_numbered_line(&pager_render.rendered.output, 5));
        assert!(contains_numbered_line(&pager_render.source.output, 5));

        let separator = line_numbers.is_some_and(|options| options.separator);
        assert_eq!(pager_render.rendered.output.contains(" │ "), separator);
        assert_eq!(pager_render.source.output.contains(" │ "), separator);
    }
}
