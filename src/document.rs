use crate::cli::{LineNumberOptions, LineNumberTarget};
use crate::config::Config;
use crate::markdown::{MarkdownProcessor, ParsedDocument};
use crate::pager;
use crate::renderer::TerminalRenderer;
use crate::terminal::OutputStyle;
use anyhow::Result;
use std::borrow::Cow;
use std::path::Path;

#[derive(Clone, Copy, Default)]
pub(crate) struct RenderOptions<'a> {
    pub(crate) do_html: bool,
    pub(crate) show_current_theme: bool,
    pub(crate) current_preset: Option<&'a str>,
    pub(crate) add_leading_blank: bool,
    pub(crate) prepare_pager_views: bool,
}

pub(crate) fn render_document(
    content: &str,
    config: &Config,
    output_style: OutputStyle,
    options: RenderOptions<'_>,
) -> Result<pager::RenderedOutput> {
    let document = parse_document(content, config, options)?;
    let renderer = TerminalRenderer::new(config, output_style)?;
    render_parsed_document(document, config, Cow::Owned(renderer), options)
}

pub(crate) fn render_document_with_renderer(
    content: &str,
    config: &Config,
    renderer: &TerminalRenderer,
    options: RenderOptions<'_>,
) -> Result<pager::RenderedOutput> {
    let document = parse_document(content, config, options)?;
    render_parsed_document(document, config, Cow::Borrowed(renderer), options)
}

fn parse_document(
    content: &str,
    config: &Config,
    options: RenderOptions<'_>,
) -> Result<ParsedDocument> {
    let do_html = options.do_html;
    let prepare_pager_views = options.prepare_pager_views;
    let processor_config =
        (prepare_pager_views && !do_html && !config.source_line_numbers_enabled()).then(|| {
            let mut config = config.clone();
            config.line_numbers = Some(LineNumberOptions {
                target: LineNumberTarget::Source,
                separator: false,
            });
            config
        });
    let processor = MarkdownProcessor::new(processor_config.as_ref().unwrap_or(config))
        .with_extended_math(!do_html);
    processor.parse_document(content)
}

fn render_parsed_document(
    document: ParsedDocument,
    config: &Config,
    renderer: Cow<'_, TerminalRenderer>,
    options: RenderOptions<'_>,
) -> Result<pager::RenderedOutput> {
    let RenderOptions {
        do_html,
        show_current_theme,
        current_preset,
        add_leading_blank,
        prepare_pager_views,
    } = options;
    let output_style = renderer.output_style();
    let pager_status_bar_transparent = renderer.pager_status_bar_transparent();

    if do_html {
        return Ok(pager::RenderedOutput::new(
            renderer.to_html_document(document)?,
            pager_status_bar_transparent,
            output_style,
        ));
    }

    let mut output = String::new();
    if let Some(name) = current_preset {
        output.push('\n');
        output.push_str("Current preset: ");
        output.push_str(name);
        output.push('\n');
    }
    if show_current_theme {
        output.push_str(&format_current_themes(config));
    }
    if add_leading_blank {
        output.push('\n');
    }
    if prepare_pager_views {
        let rendered = pager::render_terminal_document(
            &renderer,
            document.clone(),
            output.clone(),
            pager_status_bar_transparent,
        )?;
        let width = config.get_terminal_width();
        let limit = config.cols.filter(|_| config.cols_from_cli);
        let renderer = renderer.into_owned();
        return Ok(rendered.with_reflow(
            std::sync::Arc::new(move |width| {
                pager::render_terminal_document(
                    &renderer.with_layout_width(width),
                    document.clone(),
                    output.clone(),
                    pager_status_bar_transparent,
                )
            }),
            width,
            limit,
        ));
    }
    output.push_str(&renderer.render_document(document)?);
    Ok(pager::RenderedOutput::new(
        output,
        pager_status_bar_transparent,
        output_style,
    ))
}

pub(crate) fn render_document_file(
    path: &Path,
    config: &Config,
    do_html: bool,
    show_current_theme: bool,
    current_preset: Option<&str>,
    output_style: OutputStyle,
    prepare_pager_views: bool,
) -> Result<pager::PagerDocument> {
    let content = read_document_source(path)?;
    let rendered = render_document(
        &content,
        config,
        output_style,
        RenderOptions {
            do_html,
            show_current_theme,
            current_preset,
            add_leading_blank: true,
            prepare_pager_views,
        },
    )?;
    Ok(rendered.into_pager_document())
}

pub(crate) fn read_document_source(path: &Path) -> Result<String> {
    let mut content = std::fs::read_to_string(path)?;
    crate::strip_leading_bom(&mut content);
    Ok(content)
}

pub(crate) fn format_current_themes(config: &Config) -> String {
    let mut result = String::new();
    result.push('\n');
    result.push_str(&format!("Current theme: {}\n", config.theme));
    result.push_str(&format!(
        "Current code theme: {}\n",
        config.code_theme.as_deref().unwrap_or(&config.theme)
    ));
    result
}
