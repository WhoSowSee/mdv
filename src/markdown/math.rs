use super::*;

#[derive(Clone, Copy)]
enum ExtendedDelimiter {
    Inline,
    Display,
}

impl ExtendedDelimiter {
    const fn closing(self) -> &'static str {
        match self {
            Self::Inline => "\\)",
            Self::Display => "\\]",
        }
    }

    const fn replacement(self) -> &'static str {
        match self {
            Self::Inline => "$",
            Self::Display => "$$",
        }
    }
}

pub(super) struct MathPlaceholders {
    dollar: String,
    pipe: String,
    edge_space: String,
}

impl MathPlaceholders {
    pub(super) fn new(content: &str) -> Self {
        let mut nonce = 0usize;
        loop {
            let prefix = format!("\u{e000}mdv-math-{nonce}\u{e001}");
            if !content.contains(&prefix) {
                return Self {
                    dollar: format!("{prefix}d\u{e002}"),
                    pipe: format!("{prefix}p\u{e002}"),
                    edge_space: format!("{prefix}s\u{e002}"),
                };
            }
            nonce = nonce.saturating_add(1);
        }
    }

    pub(super) fn restore_events(&self, events: &mut [(Event<'_>, Range<usize>)]) {
        for (event, _) in events {
            match event {
                Event::Text(text)
                | Event::Code(text)
                | Event::Html(text)
                | Event::InlineHtml(text)
                | Event::InlineMath(text)
                | Event::DisplayMath(text)
                    if text.contains(&self.dollar)
                        || text.contains(&self.pipe)
                        || text.contains(&self.edge_space) =>
                {
                    *text = self.restore_text(text).into();
                }
                _ => {}
            }
        }
    }

    fn restore_text(&self, text: &str) -> String {
        text.replace(&self.dollar, "$")
            .replace(&self.pipe, "|")
            .replace(&self.edge_space, " ")
    }
}

impl MarkdownProcessor {
    pub(super) fn normalize_extended_math_delimiters(
        &self,
        content: &str,
        placeholders: &MathPlaceholders,
    ) -> String {
        if !content.contains("\\(") && !content.contains("\\[") {
            return self.protect_table_math_pipes(content, placeholders);
        }
        let (protected, containers) = self.extended_math_ranges(content);
        let mut output = String::with_capacity(content.len());
        let mut cursor = 0usize;

        while cursor < content.len() {
            let Some((opening, delimiter)) = next_opening(content, cursor, &protected) else {
                output.push_str(&content[cursor..]);
                break;
            };
            let content_start = opening + 2;
            let Some(closing) =
                find_closing(content, content_start, delimiter, &protected, &containers)
            else {
                output.push_str(&content[cursor..content_start]);
                cursor = content_start;
                continue;
            };

            output.push_str(&content[cursor..opening]);
            output.push_str(delimiter.replacement());
            output.push_str(&protect_markdown_syntax(
                &content[content_start..closing],
                placeholders,
            ));
            output.push_str(delimiter.replacement());
            cursor = closing + 2;
        }

        self.protect_table_math_pipes(&output, placeholders)
    }

    fn protect_table_math_pipes(&self, content: &str, placeholders: &MathPlaceholders) -> String {
        if !content.contains('$') || !content.contains('|') {
            return content.to_string();
        }
        let math_ranges = dollars::dollar_math_ranges(content, self.options);
        dollars::protect_math_pipes(content, &math_ranges, placeholders)
    }
}

fn protect_markdown_syntax(content: &str, placeholders: &MathPlaceholders) -> String {
    let mut protected = String::with_capacity(content.len());
    for (index, character) in content.char_indices() {
        match character {
            '$' if !is_escaped(content, index) => protected.push_str(&placeholders.dollar),
            '|' => protected.push_str(&placeholders.pipe),
            _ => protected.push(character),
        }
    }
    let trimmed_start = protected.trim_start_matches([' ', '\t']);
    let leading = protected.len() - trimmed_start.len();
    let trimmed = trimmed_start.trim_end_matches([' ', '\t']);
    let trailing = trimmed_start.len() - trimmed.len();
    if leading == 0 && trailing == 0 {
        return protected;
    }

    let mut output = String::with_capacity(protected.len());
    if leading > 0 {
        output.push_str(&placeholders.edge_space.repeat(leading));
    }
    output.push_str(trimmed);
    if trailing > 0 {
        output.push_str(&placeholders.edge_space.repeat(trailing));
    }
    output
}

fn next_opening(
    content: &str,
    mut cursor: usize,
    protected: &[Range<usize>],
) -> Option<(usize, ExtendedDelimiter)> {
    while let Some(relative) = content[cursor..].find('\\') {
        let index = cursor + relative;
        if is_protected(index, protected) || is_escaped(content, index) {
            cursor = index + 1;
            continue;
        }

        let delimiter = match content.as_bytes().get(index + 1) {
            Some(b'(') => ExtendedDelimiter::Inline,
            Some(b'[') => ExtendedDelimiter::Display,
            _ => {
                cursor = index + 1;
                continue;
            }
        };
        return Some((index, delimiter));
    }
    None
}

fn find_closing(
    content: &str,
    mut cursor: usize,
    delimiter: ExtendedDelimiter,
    protected: &[Range<usize>],
    containers: &[Range<usize>],
) -> Option<usize> {
    let formula_start = cursor;
    while let Some(relative) = content[cursor..].find(delimiter.closing()) {
        let index = cursor + relative;
        if is_protected(index, protected) || is_escaped(content, index) {
            cursor = index + 1;
            continue;
        }
        if matches!(delimiter, ExtendedDelimiter::Inline)
            && crosses_blank_line(&content[..index], formula_start)
        {
            return None;
        }
        let opening = formula_start.saturating_sub(2);
        if !containers
            .iter()
            .any(|range| range.start <= opening && index + 2 <= range.end)
        {
            return None;
        }
        return Some(index);
    }
    None
}

fn crosses_blank_line(content: &str, start: usize) -> bool {
    content[start..].lines().any(|line| {
        MarkdownProcessor::split_blockquote_prefix(line)
            .1
            .trim()
            .is_empty()
    })
}

fn is_protected(index: usize, ranges: &[Range<usize>]) -> bool {
    ranges
        .iter()
        .take_while(|range| range.start <= index)
        .any(|range| range.contains(&index))
}

pub(super) fn is_escaped(content: &str, index: usize) -> bool {
    let preceding = content.as_bytes()[..index]
        .iter()
        .rev()
        .take_while(|byte| **byte == b'\\')
        .count();
    preceding % 2 == 1
}

mod dollars;
mod protected;

#[cfg(test)]
mod tests {
    use super::*;

    fn normalize(processor: &MarkdownProcessor, input: &str) -> String {
        let placeholders = MathPlaceholders::new(input);
        placeholders
            .restore_text(&processor.normalize_extended_math_delimiters(input, &placeholders))
    }

    fn math_values(processor: &MarkdownProcessor, input: &str) -> Vec<String> {
        let placeholders = MathPlaceholders::new(input);
        let normalized = processor.normalize_extended_math_delimiters(input, &placeholders);
        let mut events = Parser::new_ext(&normalized, processor.options)
            .into_offset_iter()
            .collect::<Vec<_>>();
        placeholders.restore_events(&mut events);
        events
            .into_iter()
            .filter_map(|(event, _)| match event {
                Event::InlineMath(math) | Event::DisplayMath(math) => Some(math.to_string()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn extended_delimiters_skip_code_and_unclosed_input() {
        let processor = MarkdownProcessor::new(&Config::default());
        let input =
            "\\(x_1\\) `\\(code\\)` <code>\\(html\\)</code>\n\n```text\n\\[code\\]\n```\n\n\\[open";

        assert_eq!(
            normalize(&processor, input),
            "$x_1$ `\\(code\\)` <code>\\(html\\)</code>\n\n```text\n\\[code\\]\n```\n\n\\[open"
        );
        assert!(
            processor
                .normalize_explicit_blank_lines("\\[open\ntail\\")
                .contains(BLANK_LINE_MARKER)
        );
        assert_eq!(normalize(&processor, "\\(x\n- y\\)"), "\\(x\n- y\\)");
    }

    #[test]
    fn extended_delimiters_render_in_link_text_but_not_destinations() {
        let processor = MarkdownProcessor::new(&Config::default());
        let input =
            "[\\(x_1\\)](https://example.com/\\(raw\\))\n\n[id]: https://example.com/\\[raw\\]\n";

        assert_eq!(
            normalize(&processor, input),
            "[$x_1$](https://example.com/\\(raw\\))\n\n[id]: https://example.com/\\[raw\\]\n"
        );
    }

    #[test]
    fn extended_delimiters_skip_heading_attributes() {
        let processor = MarkdownProcessor::new(&Config::default());
        let input = "# \\(x\\) {#heading data-formula=\"\\(raw\\)\"}";

        assert_eq!(
            normalize(&processor, input),
            "# $x$ {#heading data-formula=\"\\(raw\\)\"}"
        );
    }

    #[test]
    fn nested_delimiters_remain_in_the_outer_formula() {
        let processor = MarkdownProcessor::new(&Config::default());

        assert_eq!(math_values(&processor, "\\(x+$y$\\)"), ["x+$y$"]);
        assert_eq!(math_values(&processor, "$x+\\(y\\)$"), ["x+\\(y\\)"]);
        assert_eq!(
            math_values(&processor, "$x+\\(y\\)+|z|$"),
            ["x+\\(y\\)+|z|"]
        );
        assert_eq!(
            math_values(&processor, "\\(\\unknown{|$|}\\)"),
            ["\\unknown{|$|}"]
        );
        let collision = "\u{e000}mdv-math-0\u{e001} \\(x\\)";
        assert_eq!(
            normalize(&processor, collision),
            "\u{e000}mdv-math-0\u{e001} $x$"
        );
    }

    #[test]
    fn dollar_math_pipes_are_protected_without_matching_currency() {
        let processor = MarkdownProcessor::new(&Config::default());

        let input = "| Formula | Price |\n| --- | --- |\n| $|x|$ | $5 | $10 |";
        let placeholders = MathPlaceholders::new(input);
        let normalized = processor.normalize_extended_math_delimiters(input, &placeholders);
        assert!(normalized.contains(&placeholders.pipe));
        assert!(normalized.contains("$5 | $10"));

        let mut events = Parser::new_ext(&normalized, processor.options)
            .into_offset_iter()
            .collect::<Vec<_>>();
        placeholders.restore_events(&mut events);
        assert!(
            events.into_iter().any(
                |(event, _)| matches!(event, Event::InlineMath(math) if math.as_ref() == "|x|")
            )
        );
    }

    #[test]
    fn reference_protection_stops_at_the_actual_destination() {
        let processor = MarkdownProcessor::new(&Config::default());
        assert_eq!(
            normalize(
                &processor,
                "[label]: https://example.com \\(\\alpha\\) prose"
            ),
            "[label]: https://example.com $\\alpha$ prose"
        );
    }

    #[test]
    fn extended_display_math_restores_edge_placeholders() {
        let processor = MarkdownProcessor::new(&Config::default());
        let input = "> [!NOTE]\n>\n> \\[\n> \\frac{a}{b}\n> \\]\n";
        let document = processor
            .with_extended_math(true)
            .parse_document(input)
            .expect("parse document");
        assert!(
            document.events.iter().any(|event| {
                matches!(event, Event::DisplayMath(source) if source.trim() == r"\frac{a}{b}")
            }),
            "{:?}",
            document.events
        );
        assert!(
            !document.events.iter().any(|event| match event {
                Event::Text(text)
                | Event::Code(text)
                | Event::Html(text)
                | Event::InlineHtml(text)
                | Event::InlineMath(text)
                | Event::DisplayMath(text) => text.contains("mdv-math-"),
                _ => false,
            }),
            "{:?}",
            document.events
        );
    }
}
