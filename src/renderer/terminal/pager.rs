use super::TerminalRenderer;
use crate::cli::{LineNumberOptions, LineNumberTarget};
use anyhow::Result;
use pulldown_cmark::Event;

pub(crate) struct PagerRender {
    pub(crate) output: String,
    pub(crate) source_lines: Vec<Option<usize>>,
    pub(crate) source_view: SourceNumberedView,
}

pub(crate) enum SourceNumberedView {
    Current,
    Alternate {
        output: String,
        source_lines: Vec<Option<usize>>,
    },
}

impl TerminalRenderer {
    pub(crate) fn render_for_pager(&self, events: Vec<Event<'static>>) -> Result<PagerRender> {
        let source_options = LineNumberOptions {
            target: LineNumberTarget::Source,
            separator: self
                .config
                .line_numbers
                .is_some_and(|options| options.separator),
        };
        if self.config.line_numbers == Some(source_options) {
            let source = self.render_with_options(events, Some(source_options), true)?;
            return Ok(PagerRender {
                output: source.output,
                source_lines: source.source_lines,
                source_view: SourceNumberedView::Current,
            });
        }

        let output = self.render_with_options(events.clone(), self.config.line_numbers, true)?;
        let source = self.render_with_options(events, Some(source_options), true)?;
        Ok(PagerRender {
            output: output.output,
            source_lines: output.source_lines,
            source_view: SourceNumberedView::Alternate {
                output: source.output,
                source_lines: source.source_lines,
            },
        })
    }
}
