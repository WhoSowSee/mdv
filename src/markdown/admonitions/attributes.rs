pub(super) fn bracketed(input: &str, open: char, close: char) -> Option<(&str, &str)> {
    let rest = input.strip_prefix(open)?;
    let mut depth = 1usize;
    let mut escaped = false;
    let mut quote = None;
    for (index, ch) in rest.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if open == '{' {
            if quote == Some(ch) {
                quote = None;
                continue;
            }
            if quote.is_some() {
                continue;
            }
            if matches!(ch, '\'' | '"') {
                quote = Some(ch);
                continue;
            }
        }
        if ch == open {
            depth += 1;
        }
        if ch == close {
            depth -= 1;
            if depth == 0 {
                return Some((&rest[..index], &rest[index + ch.len_utf8()..]));
            }
        }
    }
    None
}

pub(super) fn quoted(input: &str) -> Option<(String, &str)> {
    let quote = input.chars().next().filter(|ch| matches!(ch, '\'' | '"'))?;
    let rest = &input[1..];
    let mut escaped = false;
    let mut value = String::new();
    for (index, ch) in rest.char_indices() {
        if escaped {
            if ch != quote && ch != '\\' {
                value.push('\\');
            }
            value.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == quote {
            return Some((value, &rest[index + 1..]));
        } else {
            value.push(ch);
        }
    }
    None
}

pub(super) fn parse(mut input: &str) -> Option<Vec<(String, String)>> {
    let mut result = Vec::new();
    while !input.trim().is_empty() {
        input = input.trim_start();
        let end = input
            .find(|ch: char| ch.is_whitespace() || ch == '=')
            .unwrap_or(input.len());
        if end == 0 {
            return None;
        }
        let key = &input[..end];
        input = input[end..].trim_start();
        if let Some(rest) = input.strip_prefix('=') {
            input = rest.trim_start();
            let value;
            if input.starts_with(['\'', '"']) {
                (value, input) = quoted(input)?;
            } else {
                let end = input.find(char::is_whitespace).unwrap_or(input.len());
                if end == 0 {
                    return None;
                }
                value = input[..end].to_string();
                input = &input[end..];
            }
            result.push((key.to_string(), value));
        } else if let Some(class) = key.strip_prefix('.') {
            result.push(("class".to_string(), class.to_string()));
        } else if let Some(id) = key.strip_prefix('#') {
            result.push(("id".to_string(), id.to_string()));
        } else {
            result.push((key.to_string(), String::new()));
        }
    }
    Some(result)
}
