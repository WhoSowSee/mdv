use super::{Admonitions, Line, MarkdownProcessor, scanner};

pub(super) fn convert_container(
    lines: &[Line],
    start: usize,
    context: &mut Admonitions,
) -> Option<(Vec<Line>, usize)> {
    let before = context.metadata.len();
    if quote_body(&lines[start].text).is_some() {
        let mut end = start;
        let mut body = Vec::new();
        while let Some(line) = lines.get(end) {
            let Some(text) = quote_body(&line.text) else {
                break;
            };
            body.push(line.with_text(text));
            end += 1;
        }
        let converted = scanner::convert(&body, context);
        if before == context.metadata.len() {
            return Some((lines[start..end].to_vec(), end));
        }
        return Some((
            converted
                .into_iter()
                .map(|line| line.with_text(format!("> {}", line.text)))
                .collect(),
            end,
        ));
    }
    let marker_end = list_marker(&lines[start].text)?;
    let mut end = start + 1;
    while let Some(line) = lines.get(end) {
        if !line.text.trim().is_empty()
            && MarkdownProcessor::leading_indent_columns(&line.text) < marker_end
        {
            break;
        }
        end += 1;
    }
    let mut body = vec![lines[start].with_text(&lines[start].text[marker_end..])];
    body.extend(
        lines[start + 1..end]
            .iter()
            .map(|line| line.with_text(scanner::dedent(&line.text, marker_end))),
    );
    let converted = scanner::convert(&body, context);
    if before == context.metadata.len() {
        return Some((lines[start..end].to_vec(), end));
    }
    let prefix = &lines[start].text[..marker_end];
    Some((
        converted
            .into_iter()
            .enumerate()
            .map(|(index, line)| {
                let padding = if index == 0 {
                    prefix.to_string()
                } else {
                    " ".repeat(marker_end)
                };
                line.with_text(format!("{padding}{}", line.text))
            })
            .collect(),
        end,
    ))
}

fn quote_body(text: &str) -> Option<&str> {
    if MarkdownProcessor::leading_indent_columns(text) > 3 {
        return None;
    }
    let rest = text.trim_start().strip_prefix('>')?;
    Some(rest.strip_prefix(' ').unwrap_or(rest))
}

fn list_marker(text: &str) -> Option<usize> {
    let trimmed = text.trim_start_matches(' ');
    let indent = text.len() - trimmed.len();
    if indent > 3 {
        return None;
    }
    let marker_len = if trimmed.starts_with(['-', '+', '*']) {
        1
    } else {
        let digits = trimmed.bytes().take_while(u8::is_ascii_digit).count();
        if !(1..=9).contains(&digits) || !trimmed[digits..].starts_with(['.', ')']) {
            return None;
        }
        digits + 1
    };
    if !trimmed[marker_len..].starts_with(' ') {
        return None;
    }
    let spaces = trimmed[marker_len..]
        .bytes()
        .take_while(|byte| *byte == b' ')
        .count();
    Some(indent + marker_len + if spaces <= 4 { spaces } else { 1 })
}
