use super::{
    Admonitions, Line, MarkdownProcessor, Terminator, boundaries, containers,
    protected::ProtectedLines,
};

pub(super) fn convert(lines: &[Line], context: &mut Admonitions) -> Vec<Line> {
    let protected = ProtectedLines::new(lines);
    let mut blocks = boundaries::index(lines, &protected);
    let mut output = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        if let Some(end) = protected.end_at(index) {
            output.extend_from_slice(&lines[index..end]);
            index = end;
            continue;
        }
        let text = &lines[index].text;
        let indent = MarkdownProcessor::leading_indent_columns(text);
        if indent <= 3 {
            if let Some((mut opening, (end, next))) = blocks[index].take() {
                let body_indent = if matches!(opening.terminator, Terminator::Indent)
                    && lines[index + 1..end]
                        .iter()
                        .find(|line| !line.text.trim().is_empty())
                        .is_some_and(|line| {
                            MarkdownProcessor::leading_indent_columns(&line.text) >= indent + 4
                        }) {
                    indent + 4
                } else {
                    indent
                };
                let mut body: Vec<_> = lines[index + 1..end]
                    .iter()
                    .map(|line| line.with_text(dedent(&line.text, body_indent)))
                    .collect();
                opening.metadata.consume_options(opening.dialect, &mut body);
                let marker = context.marker(&opening.metadata);
                let converted = convert(&body, context);
                let padding = " ".repeat(indent);
                if output
                    .last()
                    .is_some_and(|line: &Line| !line.text.trim().is_empty())
                {
                    output.push(Line::blank());
                }
                output.push(lines[index].with_text(format!("{padding}> {marker}")));
                output.push(Line {
                    text: format!("{padding}>"),
                    source: None,
                });
                for line in converted {
                    output.push(line.with_text(format!("{padding}> {}", line.text)));
                }
                output.push(Line::blank());
                index = next;
                continue;
            }
            if let Some((converted, next)) = containers::convert_container(lines, index, context) {
                output.extend(converted);
                index = next;
                continue;
            }
        }
        output.push(lines[index].clone());
        index += 1;
    }
    output
}

pub(super) fn dedent(text: &str, columns: usize) -> String {
    let mut removed = 0;
    let mut end = 0;
    for (index, ch) in text.char_indices() {
        if removed >= columns || !matches!(ch, ' ' | '\t') {
            break;
        }
        removed += if ch == '\t' { 4 - removed % 4 } else { 1 };
        end = index + ch.len_utf8();
    }
    format!(
        "{}{}",
        " ".repeat(removed.saturating_sub(columns)),
        &text[end..]
    )
}
