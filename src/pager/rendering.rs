use super::{PagerContent, PagerDisplay, PagerDocument, PagerLineNumberMode, PagerLineNumberViews};
use crate::cli::LineNumberTarget;
use crate::markdown::ParsedDocument;
use crate::renderer::TerminalRenderer;
use crate::renderer::terminal::PagerRenderView;
use anyhow::Result;

pub(crate) struct RenderedOutput {
    content: PagerContent,
    status_bar_transparent: bool,
}

impl RenderedOutput {
    pub(crate) fn new(output: String, status_bar_transparent: bool) -> Self {
        Self {
            content: PagerContent::Static(output),
            status_bar_transparent,
        }
    }

    fn for_pager(views: PagerLineNumberViews, status_bar_transparent: bool) -> Self {
        Self {
            content: PagerContent::LineNumbers(views),
            status_bar_transparent,
        }
    }

    pub(crate) fn output(&self) -> &str {
        self.content.output()
    }

    pub(crate) fn into_pager_document(self, source: String) -> PagerDocument {
        PagerDocument::from_content(self.content, source)
            .with_status_bar_transparent(self.status_bar_transparent)
    }
}

pub(crate) fn render_terminal_document(
    renderer: &TerminalRenderer,
    document: ParsedDocument,
    prefix: String,
    status_bar_transparent: bool,
) -> Result<RenderedOutput> {
    let prefix_lines = prefix.lines().count();
    let rendered = renderer.render_document_for_pager(document)?;
    let mode = match rendered.initial_target {
        None => PagerLineNumberMode::Off,
        Some(LineNumberTarget::Rendered) => PagerLineNumberMode::Rendered,
        Some(LineNumberTarget::Source) => PagerLineNumberMode::Source,
    };
    let views = PagerLineNumberViews::new(
        mode,
        prefixed_display(&prefix, prefix_lines, rendered.unnumbered),
        prefixed_display(&prefix, prefix_lines, rendered.rendered),
        prefixed_display(&prefix, prefix_lines, rendered.source),
    );
    Ok(RenderedOutput::for_pager(views, status_bar_transparent))
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
