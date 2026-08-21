use super::*;
use crate::cli::FrontMatterMode;
use crate::error::MdvError;

impl MarkdownProcessor {
    pub fn new(config: &Config) -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
        options.insert(Options::ENABLE_MATH);
        options.insert(Options::ENABLE_DEFINITION_LIST);

        Self {
            config: config.clone(),
            options,
        }
    }

    /// Parse Markdown body events, validating and omitting recognized front matter.
    pub fn parse(&self, markdown: &str) -> Result<Vec<Event<'static>>> {
        Ok(self.parse_document(markdown)?.events)
    }

    pub(crate) fn parse_document(&self, markdown: &str) -> Result<ParsedDocument> {
        let document = if matches!(self.config.front_matter, FrontMatterMode::Source) {
            without_front_matter(markdown)
        } else {
            split_front_matter(markdown)?
        };
        let (content, source_lines) =
            self.preprocess_content(document.body, document.body_start_line)?;
        let parser = Parser::new_ext(&content, self.options).into_offset_iter();

        let events: Vec<(Event, Range<usize>)> = parser.collect();
        let line_starts = source_lines
            .as_ref()
            .map(|_| source_lines::starts(&content))
            .unwrap_or_default();
        let source_lines = source_lines.unwrap_or_default();
        let events = self.postprocess_events(&content, events, &line_starts, &source_lines)?;
        let events = if self.config.reverse {
            self.reverse_events(events)
        } else {
            events
        };

        Ok(ParsedDocument {
            events,
            front_matter: document.front_matter,
        })
    }

    pub(super) fn preprocess_content(
        &self,
        content: &str,
        first_source_line: usize,
    ) -> Result<(String, Option<Vec<Option<usize>>>)> {
        let mut processed = content.to_string();
        let mut source_lines = self.config.source_line_numbers_enabled().then(|| {
            (first_source_line..first_source_line + content.lines().count())
                .map(Some)
                .collect::<Vec<_>>()
        });

        if let Some(from_text) = &self.config.from_text {
            let lines: Vec<&str> = processed.lines().collect();
            let range = Self::filter_line_range(&lines, from_text);
            processed = lines[range.clone()].join("\n");
            if let Some(source_lines) = source_lines.as_mut() {
                *source_lines = source_lines[range].to_vec();
            }
        }

        processed = source_lines::apply_transform(processed, source_lines.as_mut(), |content| {
            self.normalize_tab_indented_fences(content)
        });
        processed = source_lines::apply_transform(processed, source_lines.as_mut(), |content| {
            self.normalize_explicit_blank_lines(content)
        });
        processed = source_lines::apply_transform(processed, source_lines.as_mut(), |content| {
            self.ensure_task_list_termination(content)
        });
        if self.config.pretty_checkbox.is_some() {
            processed = source_lines::apply_transform(
                processed,
                source_lines.as_mut(),
                Self::normalize_backslash_checkbox,
            );
        }
        processed = source_lines::apply_transform(processed, source_lines.as_mut(), |content| {
            self.convert_admonitions_to_callouts(content)
        });
        processed = source_lines::apply_transform(processed, source_lines.as_mut(), |content| {
            self.separate_callout_markers_from_setext(content)
        });
        processed = source_lines::apply_transform(processed, source_lines.as_mut(), |content| {
            self.preprocess_blockquotes(content)
        });

        Ok((processed, source_lines))
    }

    pub(super) fn filter_line_range(lines: &[&str], from_text: &str) -> Range<usize> {
        let (search_text, max_lines) = from_text
            .split_once(':')
            .map_or((from_text, None), |(text, lines)| {
                (text, lines.parse::<usize>().ok())
            });
        let start = if search_text.is_empty() {
            0
        } else {
            lines
                .iter()
                .position(|line| line.contains(search_text))
                .unwrap_or(0)
        };
        let end = max_lines.map_or(lines.len(), |count| {
            start.saturating_add(count).min(lines.len())
        });
        start..end
    }
}

struct SplitDocument<'a> {
    body: &'a str,
    body_start_line: usize,
    front_matter: Option<FrontMatter>,
}

fn split_front_matter(source: &str) -> Result<SplitDocument<'_>> {
    let mut lines = source.split_inclusive('\n');
    let Some(opening) = lines.next() else {
        return Ok(without_front_matter(source));
    };
    if line_content(opening) != "---" {
        return Ok(without_front_matter(source));
    }

    let raw_start = opening.len();
    let mut line_start = raw_start;
    for (line_index, line) in lines.enumerate() {
        if line_content(line) == "---" {
            let raw = &source[raw_start..line_start];
            if raw.trim().is_empty() {
                return Ok(without_front_matter(source));
            }

            let value = serde_yaml::from_str(raw).map_err(front_matter_error)?;
            let serde_yaml::Value::Mapping(properties) = value else {
                return Ok(without_front_matter(source));
            };
            let body_start = line_start + line.len();
            return Ok(SplitDocument {
                body: &source[body_start..],
                body_start_line: line_index + 3,
                front_matter: Some(FrontMatter {
                    raw: raw.to_string(),
                    properties,
                }),
            });
        }
        line_start += line.len();
    }

    Ok(without_front_matter(source))
}

fn without_front_matter(source: &str) -> SplitDocument<'_> {
    SplitDocument {
        body: source,
        body_start_line: 1,
        front_matter: None,
    }
}

fn line_content(line: &str) -> &str {
    let line = line.strip_suffix('\n').unwrap_or(line);
    line.strip_suffix('\r').unwrap_or(line)
}

fn front_matter_error(error: serde_yaml::Error) -> MdvError {
    let message = match error.location() {
        Some(location) => format!(
            "invalid YAML front matter at document line {}, column {}: {error}",
            location.line() + 1,
            location.column()
        ),
        None => format!("invalid YAML front matter: {error}"),
    };
    MdvError::MarkdownError(message)
}
