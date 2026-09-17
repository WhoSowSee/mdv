use crate::cli::{LineNumberOptions, LineNumberTarget, OutputStyle};
use crate::config::Config;
use crate::markdown::MarkdownProcessor;
use crate::pager;
use crate::renderer::TerminalRenderer;
use anyhow::Result;
use std::path::Path;

#[derive(Default)]
pub(crate) struct RenderOptions<'a> {
    pub(crate) do_html: bool,
    pub(crate) show_current_theme: bool,
    pub(crate) current_preset: Option<&'a str>,
    pub(crate) add_leading_blank: bool,
    pub(crate) for_pager: bool,
}

pub(crate) fn render_document(
    content: &str,
    config: &Config,
    output_style: OutputStyle,
    options: RenderOptions<'_>,
) -> Result<pager::RenderedOutput> {
    let RenderOptions {
        do_html,
        show_current_theme,
        current_preset,
        add_leading_blank,
        for_pager,
    } = options;
    let processor_config =
        (for_pager && !do_html && !config.source_line_numbers_enabled()).then(|| {
            let mut config = config.clone();
            config.line_numbers = Some(LineNumberOptions {
                target: LineNumberTarget::Source,
                separator: false,
            });
            config
        });
    let processor = MarkdownProcessor::new(processor_config.as_ref().unwrap_or(config))
        .with_extended_math(!do_html);
    let document = processor.parse_document(content)?;
    let renderer = TerminalRenderer::new(config, output_style)?;
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
    if for_pager {
        return pager::render_terminal_document(
            &renderer,
            document,
            output,
            pager_status_bar_transparent,
        );
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
) -> Result<pager::PagerDocument> {
    let mut content = std::fs::read_to_string(path)?;
    crate::strip_leading_bom(&mut content);
    let rendered = render_document(
        &content,
        config,
        output_style,
        RenderOptions {
            do_html,
            show_current_theme,
            current_preset,
            add_leading_blank: true,
            for_pager: true,
        },
    )?;
    Ok(rendered.into_pager_document(content))
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
