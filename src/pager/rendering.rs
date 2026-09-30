use super::{PagerContent, PagerDisplay, PagerDocument, PagerLineNumberMode, PagerLineNumberViews};
use crate::cli::LineNumberTarget;
use crate::markdown::ParsedDocument;
use crate::renderer::TerminalRenderer;
use crate::renderer::terminal::PagerRenderView;
use crate::terminal::OutputStyle;
use anyhow::Result;

pub(crate) type Reflow = std::sync::Arc<dyn Fn(usize) -> Result<RenderedOutput> + Send + Sync>;

pub(crate) struct RenderedOutput {
    reflow: Option<Reflow>,
    layout_width: Option<usize>,
    width_limit: Option<usize>,
    content: PagerContent,
    status_bar_transparent: bool,
    output_style: OutputStyle,
}

impl RenderedOutput {
    pub(crate) fn new(
        output: String,
        status_bar_transparent: bool,
        output_style: OutputStyle,
    ) -> Self {
        Self {
            reflow: None,
            layout_width: None,
            width_limit: None,
            content: PagerContent::Static(output),
            status_bar_transparent,
            output_style,
        }
    }

    fn for_pager(
        views: PagerLineNumberViews,
        status_bar_transparent: bool,
        output_style: OutputStyle,
    ) -> Self {
        Self {
            reflow: None,
            layout_width: None,
            width_limit: None,
            content: PagerContent::LineNumbers(views),
            status_bar_transparent,
            output_style,
        }
    }

    pub(crate) fn output(&self) -> Result<&str> {
        self.content.output()
    }

    pub(in crate::pager) fn into_content(self) -> PagerContent {
        self.content
    }

    pub(crate) fn into_pager_document(self) -> PagerDocument {
        PagerDocument::from_content(self.content, self.output_style)
            .with_status_bar_transparent(self.status_bar_transparent)
            .with_reflow(self.reflow, self.layout_width, self.width_limit)
    }

    pub(crate) fn with_reflow(
        mut self,
        reflow: Reflow,
        width: usize,
        limit: Option<usize>,
    ) -> Self {
        self.reflow = Some(reflow);
        self.layout_width = Some(width);
        self.width_limit = limit;
        self
    }
}

pub(crate) fn render_terminal_document(
    renderer: &TerminalRenderer,
    document: ParsedDocument,
    prefix: String,
    status_bar_transparent: bool,
) -> Result<RenderedOutput> {
    let prefix_lines = prefix.lines().count();
    let toc = super::toc::headings(&document.events);
    let rendered = renderer.render_document_for_pager(document)?;
    let mode = match rendered.initial_target {
        None => PagerLineNumberMode::Off,
        Some(LineNumberTarget::Rendered) => PagerLineNumberMode::Rendered,
        Some(LineNumberTarget::Source) => PagerLineNumberMode::Source,
    };
    let views = PagerLineNumberViews::new(
        mode,
        std::sync::Arc::new(move |mode| {
            let target = match mode {
                PagerLineNumberMode::Off => None,
                PagerLineNumberMode::Rendered => Some(LineNumberTarget::Rendered),
                PagerLineNumberMode::Source => Some(LineNumberTarget::Source),
            };
            let view = (rendered.render_view)(target)?;
            Ok(prefixed_display(&prefix, prefix_lines, view))
        }),
    )
    .with_toc(toc);
    Ok(RenderedOutput::for_pager(
        views,
        status_bar_transparent,
        renderer.output_style(),
    ))
}

fn prefixed_display(prefix: &str, prefix_lines: usize, rendered: PagerRenderView) -> PagerDisplay {
    let mut output = String::with_capacity(prefix.len() + rendered.output.len());
    output.push_str(prefix);
    output.push_str(&rendered.output);
    PagerDisplay::new(
        output,
        with_unmapped_prefix(prefix_lines, rendered.source_lines),
    )
}

fn with_unmapped_prefix(
    prefix_lines: usize,
    source_lines: Vec<Option<usize>>,
) -> Vec<Option<usize>> {
    std::iter::repeat_n(None, prefix_lines)
        .chain(source_lines)
        .collect()
}
