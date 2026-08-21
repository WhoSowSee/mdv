use super::terminal::TerminalRenderer;
use crate::cli::FrontMatterMode;
use crate::config::Config;
use crate::markdown::{FrontMatter, MarkdownProcessor, ParsedDocument};
use crate::utils::escape_html_text;
use anyhow::Result;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};
use serde_yaml::{Mapping, Value};

impl TerminalRenderer {
    pub(crate) fn render_document(&self, document: ParsedDocument) -> Result<String> {
        self.render(document_events(document, &self.config, false)?)
    }

    pub(crate) fn to_html_document(&self, document: ParsedDocument) -> Result<String> {
        self.to_html(document_events(document, &self.config, true)?)
    }
}

fn document_events(
    mut document: ParsedDocument,
    config: &Config,
    html: bool,
) -> Result<Vec<Event<'static>>> {
    let Some(front_matter) = document.front_matter.as_ref() else {
        return Ok(document.events);
    };
    let mut metadata = match config.front_matter {
        FrontMatterMode::Hidden | FrontMatterMode::Source => return Ok(document.events),
        FrontMatterMode::Panel if html => {
            vec![Event::Html(render_panel_html(front_matter)?.into())]
        }
        FrontMatterMode::Panel => render_panel_events(front_matter, config)?,
        FrontMatterMode::Table => table_events(front_matter, config)?,
        FrontMatterMode::Plain if html => {
            vec![Event::Html(render_plain_html(front_matter)?.into())]
        }
        FrontMatterMode::Plain => joined_property_events(front_matter, config, "\n")?,
        FrontMatterMode::Inline => joined_property_events(front_matter, config, " • ")?,
        FrontMatterMode::Blocks => blocks_events(front_matter, config)?,
        FrontMatterMode::Code => code_events(front_matter),
    };

    if config.reverse {
        document.events.append(&mut metadata);
        Ok(document.events)
    } else {
        metadata.append(&mut document.events);
        Ok(metadata)
    }
}

fn render_panel_events(front_matter: &FrontMatter, config: &Config) -> Result<Vec<Event<'static>>> {
    let mut markdown = String::from("> [!properties] Properties\n");
    for (key, value) in properties(&front_matter.properties)? {
        markdown.push_str("> **");
        markdown.push_str(&escape_markdown(&key));
        markdown.push_str(":** ");
        markdown.push_str(&escape_markdown(&value));
        markdown.push('\n');
    }

    metadata_events(&markdown, config)
}

fn render_panel_html(front_matter: &FrontMatter) -> Result<String> {
    let mut html = String::from(
        "<section class=\"front-matter\"><h2>Properties</h2><dl class=\"front-matter-properties\">\n",
    );
    for (key, value) in properties(&front_matter.properties)? {
        html.push_str("<dt>");
        html.push_str(&escape_html_text(&key));
        html.push_str("</dt><dd>");
        html.push_str(&escape_html_text(&value));
        html.push_str("</dd>\n");
    }
    html.push_str("</dl></section>\n");
    Ok(html)
}

fn render_plain_html(front_matter: &FrontMatter) -> Result<String> {
    let mut html = String::from("<p class=\"front-matter-plain\">");
    for (index, (key, value)) in properties(&front_matter.properties)?
        .into_iter()
        .enumerate()
    {
        if index > 0 {
            html.push_str("<br />\n");
        }
        html.push_str("<strong>");
        html.push_str(&escape_html_text(&key));
        html.push_str(":</strong> ");
        html.push_str(&escape_html_text(&value));
    }
    html.push_str("</p>\n");
    Ok(html)
}

fn table_events(front_matter: &FrontMatter, config: &Config) -> Result<Vec<Event<'static>>> {
    let mut markdown = String::from("| Property | Value |\n| --- | --- |\n");
    for (key, value) in properties(&front_matter.properties)? {
        markdown.push_str("| ");
        markdown.push_str(&escape_markdown(&key));
        markdown.push_str(" | ");
        markdown.push_str(&escape_markdown(&value));
        markdown.push_str(" |\n");
    }

    metadata_events(&markdown, config)
}

fn joined_property_events(
    front_matter: &FrontMatter,
    config: &Config,
    separator: &str,
) -> Result<Vec<Event<'static>>> {
    let items = properties(&front_matter.properties)?
        .into_iter()
        .map(|(key, value)| format!("**{}:** {}", escape_markdown(&key), escape_markdown(&value)))
        .collect::<Vec<_>>();
    metadata_events(&items.join(separator), config)
}

fn blocks_events(front_matter: &FrontMatter, config: &Config) -> Result<Vec<Event<'static>>> {
    let blocks = properties(&front_matter.properties)?
        .into_iter()
        .map(|(key, value)| {
            format!(
                "**{}**\n: {}",
                escape_markdown(&key),
                escape_markdown(&value)
            )
        })
        .collect::<Vec<_>>();
    metadata_events(&blocks.join("\n\n"), config)
}

fn metadata_events(markdown: &str, config: &Config) -> Result<Vec<Event<'static>>> {
    let mut metadata_config = config.clone();
    metadata_config.reverse = false;
    metadata_config.from_text = None;
    metadata_config.line_numbers = None;
    MarkdownProcessor::new(&metadata_config).parse(markdown)
}

fn properties(mapping: &Mapping) -> Result<Vec<(String, String)>> {
    mapping
        .iter()
        .map(|(key, value)| Ok((yaml_text(key)?, display_value(value)?)))
        .collect()
}

fn scalar_text(value: &Value) -> Option<String> {
    match value {
        Value::Null => Some("null".to_string()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) => Some(value.to_string()),
        Value::String(value) if value.is_empty() => Some("\"\"".to_string()),
        Value::String(value) => Some(value.split_whitespace().collect::<Vec<_>>().join(" ")),
        Value::Tagged(tagged) => scalar_text(&tagged.value),
        Value::Sequence(_) | Value::Mapping(_) => None,
    }
}

fn display_value(value: &Value) -> Result<String> {
    if let Value::Sequence(sequence) = value {
        if sequence.is_empty() {
            return Ok("[]".to_string());
        }
        if let Some(values) = sequence.iter().map(scalar_text).collect::<Option<Vec<_>>>() {
            return Ok(values.join(" · "));
        }
    }
    yaml_text(value)
}

fn yaml_text(value: &Value) -> Result<String> {
    match scalar_text(value) {
        Some(value) => Ok(value),
        None => Ok(serde_yaml::to_string(value)?
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")),
    }
}

fn code_events(front_matter: &FrontMatter) -> Vec<Event<'static>> {
    vec![
        Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::Borrowed(
            "yaml",
        )))),
        Event::Text(front_matter.raw.clone().into()),
        Event::End(TagEnd::CodeBlock),
    ]
}

fn escape_markdown(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_punctuation() {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}
