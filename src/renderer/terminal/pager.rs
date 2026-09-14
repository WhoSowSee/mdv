use super::TerminalRenderer;
use crate::cli::{LineNumberOptions, LineNumberTarget};
use crate::renderer::line_numbers::SourceMappedOutput;
use anyhow::Result;
use pulldown_cmark::Event;

pub(crate) struct PagerRender {
    pub(crate) initial_target: Option<LineNumberTarget>,
    pub(crate) unnumbered: PagerRenderView,
    pub(crate) rendered: PagerRenderView,
    pub(crate) source: PagerRenderView,
}

pub(crate) struct PagerRenderView {
    pub(crate) output: String,
    pub(crate) source_lines: Vec<Option<usize>>,
}

impl From<SourceMappedOutput> for PagerRenderView {
    fn from(rendered: SourceMappedOutput) -> Self {
        Self {
            output: rendered.output,
            source_lines: rendered.source_lines,
        }
    }
}

impl TerminalRenderer {
    pub(crate) fn render_for_pager(&self, events: Vec<Event<'static>>) -> Result<PagerRender> {
        let separator = self
            .config
            .line_numbers
            .is_some_and(|options| options.separator);
        let rendered_options = LineNumberOptions {
            target: LineNumberTarget::Rendered,
            separator,
        };
        let source_options = LineNumberOptions {
            target: LineNumberTarget::Source,
            separator,
        };

        Ok(PagerRender {
            initial_target: self.config.line_numbers.map(|options| options.target),
            unnumbered: self.render_with_options(events.clone(), None, true)?.into(),
            rendered: self
                .render_with_options(events.clone(), Some(rendered_options), true)?
                .into(),
            source: self
                .render_with_options(events, Some(source_options), true)?
                .into(),
        })
    }
}
