use crate::config::Config;
use anyhow::Result;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd};

/// Markdown processor that parses markdown and prepares it for rendering
pub struct MarkdownProcessor {
    config: Config,
    options: Options,
}

impl MarkdownProcessor {
    pub fn new(config: &Config) -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

        Self {
            config: config.clone(),
            options,
        }
    }

    pub fn parse(&self, markdown: &str) -> Result<Vec<Event<'static>>> {
        let content = self.preprocess_content(markdown)?;
        let parser = Parser::new_ext(&content, self.options);

        let events: Vec<Event> = parser.collect();
        Ok(self.postprocess_events(events)?)
    }

    fn preprocess_content(&self, content: &str) -> Result<String> {
        let mut processed = content.to_string();

        if let Some(from_text) = &self.config.from_text {
            processed = self.filter_from_text(&processed, from_text)?;
        }

        processed = self.preprocess_blockquotes(&processed);

        processed = processed.replace('\t', &" ".repeat(self.config.tab_length));

        Ok(processed)
    }

    fn filter_from_text(&self, content: &str, from_text: &str) -> Result<String> {
        // Parse from_text format: "Some Head:10" -> displays 10 lines after 'Some Head'
        let (search_text, max_lines) = if let Some((text, lines)) = from_text.split_once(':') {
            let max_lines = lines.parse::<usize>().unwrap_or(usize::MAX);
            (text, Some(max_lines))
        } else {
            (from_text, None)
        };

        let lines: Vec<&str> = content.lines().collect();

        let start_idx = if search_text.is_empty() {
            0
        } else {
            lines
                .iter()
                .position(|line| line.contains(search_text))
                .unwrap_or(0)
        };

        let end_idx = if let Some(max_lines) = max_lines {
            std::cmp::min(start_idx + max_lines, lines.len())
        } else {
            lines.len()
        };

        Ok(lines[start_idx..end_idx].join("\n"))
    }

    /// Preprocess blockquotes to ensure proper nesting behavior
    /// This fixes the issue where nested blockquotes with different levels
    /// are not properly closed by the markdown parser
    fn preprocess_blockquotes(&self, content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        let mut last_level = 0;

        for line in lines {
            let trimmed = line.trim_start();

            // Count the blockquote level (number of '>' characters)
            let mut level = 0;
            let mut chars = trimmed.chars().peekable();

            while let Some(&'>') = chars.peek() {
                chars.next();
                level += 1;
                // Skip optional space after '>'
                if let Some(&' ') = chars.peek() {
                    chars.next();
                }
            }

            // If this line has a blockquote but at a lower level than the previous,
            // add empty lines to properly close the higher levels
            if level > 0 && level < last_level {
                // Add empty lines to close the higher levels
                for _ in level..last_level {
                    result.push(String::new());
                }
            }
            // If this line has no blockquote but the previous line was a blockquote,
            // and this line is not empty, add an empty line to close the blockquote
            else if level == 0 && last_level > 0 && !trimmed.is_empty() {
                // Add empty lines to close all blockquote levels
                for _ in 0..last_level {
                    result.push(String::new());
                }
            }

            result.push(line.to_string());

            if level > 0 {
                last_level = level;
            } else if !trimmed.is_empty() {
                // Reset level when we encounter non-blockquote content
                last_level = 0;
            }
        }

        result.join("\n")
    }

    fn postprocess_events(&self, events: Vec<Event>) -> Result<Vec<Event<'static>>> {
        let mut processed = Vec::new();

        for event in events {
            match event {
                Event::Start(Tag::Heading { .. }) => {
                    processed.push(self.convert_to_static(event));
                }
                Event::End(TagEnd::Heading(_level)) => {
                    processed.push(self.convert_to_static(event));
                }
                Event::Text(text) => {
                    let processed_text = self.process_text(&text);
                    processed.push(Event::Text(processed_text.to_string().into()));
                }
                Event::Code(code) => {
                    processed.push(Event::Code(code.to_string().into()));
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    let static_kind = match kind {
                        CodeBlockKind::Indented => CodeBlockKind::Indented,
                        CodeBlockKind::Fenced(lang) => {
                            CodeBlockKind::Fenced(lang.to_string().into())
                        }
                    };
                    processed.push(Event::Start(Tag::CodeBlock(static_kind)));
                }
                _ => processed.push(self.convert_to_static(event)),
            }
        }

        Ok(processed)
    }

    fn convert_to_static(&self, event: Event) -> Event<'static> {
        match event {
            Event::Start(tag) => Event::Start(self.convert_tag_to_static(tag)),
            Event::End(tag_end) => Event::End(tag_end),
            Event::Text(text) => Event::Text(text.to_string().into()),
            Event::Code(code) => Event::Code(code.to_string().into()),
            Event::Html(html) => Event::Html(html.to_string().into()),
            Event::InlineHtml(html) => Event::InlineHtml(html.to_string().into()),
            Event::FootnoteReference(name) => Event::FootnoteReference(name.to_string().into()),
            Event::SoftBreak => Event::SoftBreak,
            Event::HardBreak => Event::HardBreak,
            Event::Rule => Event::Rule,
            Event::TaskListMarker(checked) => Event::TaskListMarker(checked),
            Event::InlineMath(math) => Event::InlineMath(math.to_string().into()),
            Event::DisplayMath(math) => Event::DisplayMath(math.to_string().into()),
        }
    }

    fn convert_tag_to_static(&self, tag: Tag) -> Tag<'static> {
        match tag {
            Tag::Paragraph => Tag::Paragraph,
            Tag::Heading {
                level,
                id,
                classes,
                attrs,
            } => Tag::Heading {
                level,
                id: id.map(|s| s.to_string().into()),
                classes: classes.into_iter().map(|s| s.to_string().into()).collect(),
                attrs: attrs
                    .into_iter()
                    .map(|(k, v)| (k.to_string().into(), v.map(|s| s.to_string().into())))
                    .collect(),
            },
            Tag::BlockQuote(kind) => Tag::BlockQuote(kind),
            Tag::CodeBlock(kind) => {
                let static_kind = match kind {
                    CodeBlockKind::Indented => CodeBlockKind::Indented,
                    CodeBlockKind::Fenced(lang) => CodeBlockKind::Fenced(lang.to_string().into()),
                };
                Tag::CodeBlock(static_kind)
            }
            Tag::List(start) => Tag::List(start),
            Tag::Item => Tag::Item,
            Tag::FootnoteDefinition(name) => Tag::FootnoteDefinition(name.to_string().into()),
            Tag::Table(alignments) => Tag::Table(alignments),
            Tag::TableHead => Tag::TableHead,
            Tag::TableRow => Tag::TableRow,
            Tag::TableCell => Tag::TableCell,
            Tag::Emphasis => Tag::Emphasis,
            Tag::Strong => Tag::Strong,
            Tag::Strikethrough => Tag::Strikethrough,
            Tag::Superscript => Tag::Superscript,
            Tag::Subscript => Tag::Subscript,
            Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            } => Tag::Link {
                link_type,
                dest_url: dest_url.to_string().into(),
                title: title.to_string().into(),
                id: id.to_string().into(),
            },
            Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            } => Tag::Image {
                link_type,
                dest_url: dest_url.to_string().into(),
                title: title.to_string().into(),
                id: id.to_string().into(),
            },
            Tag::MetadataBlock(kind) => Tag::MetadataBlock(kind),
            Tag::HtmlBlock => Tag::HtmlBlock,
            Tag::DefinitionList => Tag::DefinitionList,
            Tag::DefinitionListTitle => Tag::DefinitionListTitle,
            Tag::DefinitionListDefinition => Tag::DefinitionListDefinition,
        }
    }

    fn process_text<'a>(&self, text: &CowStr<'a>) -> CowStr<'a> {
        text.clone()
    }
}

/// Extract language from code block
pub fn extract_code_language(kind: &CodeBlockKind) -> Option<String> {
    match kind {
        CodeBlockKind::Fenced(lang) => {
            let lang = lang.trim();
            if lang.is_empty() {
                None
            } else {
                // Handle language-specific prefixes
                let lang = if lang.starts_with("language-") {
                    &lang[9..]
                } else {
                    lang
                };
                Some(lang.to_string())
            }
        }
        CodeBlockKind::Indented => None,
    }
}

/// Check if content looks like source code based on file extension or content
pub fn detect_source_code(content: &str, filename: Option<&str>) -> Option<String> {
    // Check file extension first
    if let Some(filename) = filename {
        if let Some(ext) = std::path::Path::new(filename).extension() {
            if let Some(ext_str) = ext.to_str() {
                return match ext_str.to_lowercase().as_str() {
                    "rs" => Some("rust".to_string()),
                    "py" => Some("python".to_string()),
                    "js" => Some("javascript".to_string()),
                    "ts" => Some("typescript".to_string()),
                    "go" => Some("go".to_string()),
                    "c" => Some("c".to_string()),
                    "cpp" | "cc" | "cxx" => Some("cpp".to_string()),
                    "java" => Some("java".to_string()),
                    "rb" => Some("ruby".to_string()),
                    "php" => Some("php".to_string()),
                    "sh" | "bash" => Some("bash".to_string()),
                    "sql" => Some("sql".to_string()),
                    "json" => Some("json".to_string()),
                    "yaml" | "yml" => Some("yaml".to_string()),
                    "toml" => Some("toml".to_string()),
                    "xml" => Some("xml".to_string()),
                    "html" => Some("html".to_string()),
                    "css" => Some("css".to_string()),
                    _ => None,
                };
            }
        }
    }

    // Try to detect from content patterns
    let lines: Vec<&str> = content.lines().take(10).collect();

    // Look for shebangs
    if let Some(first_line) = lines.first() {
        if first_line.starts_with("#!") {
            if first_line.contains("python") {
                return Some("python".to_string());
            } else if first_line.contains("bash") || first_line.contains("sh") {
                return Some("bash".to_string());
            } else if first_line.contains("node") {
                return Some("javascript".to_string());
            }
        }
    }

    // Look for common patterns
    for line in &lines {
        let line = line.trim();

        // Python patterns
        if line.starts_with("def ")
            || line.starts_with("class ")
            || line.starts_with("import ")
            || line.starts_with("from ")
        {
            return Some("python".to_string());
        }

        // Rust patterns
        if line.starts_with("fn ")
            || line.starts_with("struct ")
            || line.starts_with("impl ")
            || line.starts_with("use ")
        {
            return Some("rust".to_string());
        }

        // JavaScript/TypeScript patterns
        if line.starts_with("function ")
            || line.starts_with("const ")
            || line.starts_with("let ")
            || line.starts_with("var ")
        {
            return Some("javascript".to_string());
        }

        // Go patterns
        if line.starts_with("package ") || line.starts_with("func ") {
            return Some("go".to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_markdown_parsing() {
        let config = Config::default();
        let processor = MarkdownProcessor::new(&config);

        let markdown = "# Hello\n\nThis is **bold** text.";
        let events = processor.parse(markdown).unwrap();

        assert!(!events.is_empty());
    }

    #[test]
    fn test_filter_from_text() {
        let config = Config::default();
        let processor = MarkdownProcessor::new(&config);

        let content = "Line 1\nTarget Line\nLine 3\nLine 4";
        let result = processor.filter_from_text(content, "Target:2").unwrap();

        assert_eq!(result, "Target Line\nLine 3");
    }

    #[test]
    fn test_extract_code_language() {
        let fenced = CodeBlockKind::Fenced("rust".into());
        assert_eq!(extract_code_language(&fenced), Some("rust".to_string()));

        let indented = CodeBlockKind::Indented;
        assert_eq!(extract_code_language(&indented), None);
    }

    #[test]
    fn test_detect_source_code() {
        // Test file extension detection
        assert_eq!(
            detect_source_code("", Some("test.rs")),
            Some("rust".to_string())
        );
        assert_eq!(
            detect_source_code("", Some("test.py")),
            Some("python".to_string())
        );

        // Test content detection
        let python_code = "def hello():\n    print('world')";
        assert_eq!(
            detect_source_code(python_code, None),
            Some("python".to_string())
        );

        let rust_code = "fn main() {\n    println!(\"Hello\");\n}";
        assert_eq!(
            detect_source_code(rust_code, None),
            Some("rust".to_string())
        );
    }
}
