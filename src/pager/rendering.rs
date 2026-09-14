use super::PagerDocument;
use crate::markdown::ParsedDocument;
use crate::renderer::TerminalRenderer;
use crate::renderer::terminal::SourceNumberedView;
use anyhow::Result;
use minus::LineNavigation;

pub(crate) struct RenderedOutput {
    pub(crate) output: String,
    pub(crate) line_navigation: Option<LineNavigation>,
    pub(crate) status_bar_transparent: bool,
}

impl RenderedOutput {
    pub(crate) fn into_pager_document(self, source: String) -> PagerDocument {
        let mut document = PagerDocument::new(self.output, source)
            .with_status_bar_transparent(self.status_bar_transparent);
        document.line_navigation = self.line_navigation;
        document
    }
}

pub(crate) fn render_terminal_document(
    renderer: &TerminalRenderer,
    document: ParsedDocument,
    mut prefix: String,
) -> Result<(String, Option<LineNavigation>)> {
    let prefix_lines = prefix.lines().count();
    let rendered = renderer.render_document_for_pager(document)?;
    let display_source_lines = with_unmapped_prefix(prefix_lines, rendered.source_lines);

    let navigation = if display_source_lines.iter().any(Option::is_some) {
        Some(match rendered.source_view {
            SourceNumberedView::Current => LineNavigation::from_current(display_source_lines),
            SourceNumberedView::Alternate {
                output,
                source_lines,
            } => {
                let mut navigation_text = prefix.clone();
                navigation_text.push_str(&output);
                LineNavigation::new(
                    navigation_text,
                    display_source_lines,
                    with_unmapped_prefix(prefix_lines, source_lines),
                )
            }
        })
    } else {
        None
    };
    prefix.push_str(&rendered.output);
    Ok((prefix, navigation))
}

fn with_unmapped_prefix(
    prefix_lines: usize,
    source_lines: Vec<Option<usize>>,
) -> Vec<Option<usize>> {
    std::iter::repeat_n(None, prefix_lines)
        .chain(source_lines)
        .collect()
}
