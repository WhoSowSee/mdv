use super::TerminalRenderer;
use crate::cli::{LineNumberOptions, LineNumberTarget};
use crate::renderer::line_numbers::SourceMappedOutput;
use anyhow::Result;
use pulldown_cmark::Event;

pub(crate) struct PagerRender {
    pub(crate) initial_target: Option<LineNumberTarget>,
    pub(crate) render_view: PagerViewRenderer,
}

pub(crate) type PagerViewRenderer =
    std::sync::Arc<dyn Fn(Option<LineNumberTarget>) -> Result<PagerRenderView> + Send + Sync>;

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
    pub(crate) fn render_for_pager(&self, events: Vec<Event<'static>>) -> PagerRender {
        let renderer = self.clone();
        let math_diagnostics = crate::math::MathDiagnostics::default();
        let separator = self
            .config
            .line_numbers
            .is_some_and(|options| options.separator);
        PagerRender {
            initial_target: self.config.line_numbers.map(|options| options.target),
            render_view: std::sync::Arc::new(move |target| {
                let options = target.map(|target| LineNumberOptions { target, separator });
                let diagnostics = std::rc::Rc::new(math_diagnostics.clone());
                renderer
                    .render_with_options(&events, options, true, &diagnostics)
                    .map(Into::into)
            }),
        }
    }
}
