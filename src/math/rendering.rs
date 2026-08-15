use super::*;

pub(super) fn normalize_output(mut rendered: String, mode: MathMode) -> String {
    match mode {
        MathMode::Inline => {
            let mut collapsed = String::new();
            let mut prev_space = false;
            for ch in rendered.chars() {
                if ch.is_whitespace() {
                    if !prev_space {
                        collapsed.push(' ');
                        prev_space = true;
                    }
                } else {
                    collapsed.push(ch);
                    prev_space = false;
                }
            }
            collapsed.trim().to_string()
        }
        MathMode::Display => {
            let mut lines: Vec<String> = rendered
                .lines()
                .map(|line| line.trim_end().to_string())
                .collect();
            while matches!(lines.first(), Some(line) if line.is_empty()) {
                lines.remove(0);
            }
            while matches!(lines.last(), Some(line) if line.is_empty()) {
                lines.pop();
            }
            rendered = lines.join("\n");
            rendered
        }
    }
}

pub(super) fn render_text_command(command: &str, content: &str) -> String {
    if command == "mathbb"
        && let Some(symbol) = mathbb_symbol(content.trim())
    {
        return symbol.to_string();
    }
    content.to_string()
}

pub(super) fn render_fraction(numerator: &str, denominator: &str) -> String {
    let num = wrap_if_needed(numerator);
    let den = wrap_if_needed(denominator);
    format!("{}⁄{}", num, den)
}

pub(super) fn render_sqrt(index: Option<&str>, radicand: &str) -> String {
    let core = wrap_if_needed(radicand);
    if let Some(index) = index {
        let idx_rendered = render_math(index, MathMode::Inline);
        let superscript = convert_script(&idx_rendered, ScriptKind::Sup);
        format!("{}√{}", superscript, core)
    } else {
        format!("√{}", core)
    }
}

pub(super) fn render_binom(upper: &str, lower: &str) -> String {
    format!("C({},{})", upper.trim(), lower.trim())
}

pub(super) fn wrap_if_needed(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let needs_parens = trimmed
        .chars()
        .any(|ch| ch.is_whitespace() || matches!(ch, '+' | '-' | '*' | '/' | '=' | '−'));
    if needs_parens && !is_wrapped(trimmed) {
        format!("({})", trimmed)
    } else {
        trimmed.to_string()
    }
}

pub(super) fn is_wrapped(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.first() == Some(&b'(') && bytes.last() == Some(&b')')
}

pub(super) fn render_environment(env: &str, content: &str, mode: MathMode) -> String {
    let normalized = env.trim();
    if normalized.is_empty() {
        return render_math(content, mode);
    }

    let lower = normalized.to_ascii_lowercase();
    match lower.as_str() {
        "align" | "align*" | "aligned" | "eqnarray" | "split" => render_align_environment(content),
        "matrix" | "pmatrix" | "bmatrix" | "vmatrix" | "vmatrix*" | "cases" | "bmatrix*" => {
            render_matrix_environment(&lower, content)
        }
        _ => render_math(content, mode),
    }
}

pub(super) fn render_align_environment(content: &str) -> String {
    let rows = split_rows(content);
    if rows.is_empty() {
        return String::new();
    }

    let mut rendered_rows = Vec::new();
    let mut col_widths: Vec<usize> = Vec::new();

    for row in rows {
        let cols: Vec<String> = row
            .split('&')
            .map(|col| render_math(col, MathMode::Inline))
            .collect();
        if col_widths.len() < cols.len() {
            col_widths.resize(cols.len(), 0);
        }
        for (idx, col) in cols.iter().enumerate() {
            col_widths[idx] = col_widths[idx].max(display_width(col));
        }
        rendered_rows.push(cols);
    }

    let mut lines = Vec::new();
    let col_count = col_widths.len();
    for cols in rendered_rows {
        let mut line = String::new();
        for idx in 0..col_count {
            let col = cols.get(idx).map(String::as_str).unwrap_or("");
            let padding = col_widths
                .get(idx)
                .copied()
                .unwrap_or(0)
                .saturating_sub(display_width(col));
            line.push_str(col);
            if idx + 1 < col_count {
                line.push_str(&" ".repeat(padding + 1));
            }
        }
        lines.push(line.trim_end().to_string());
    }

    lines.join("\n")
}

pub(super) fn render_matrix_environment(env: &str, content: &str) -> String {
    let (left, right) = match env {
        "pmatrix" => ("(", ")"),
        "bmatrix" | "bmatrix*" => ("[", "]"),
        "vmatrix" | "vmatrix*" => ("|", "|"),
        "cases" => ("{", ""),
        _ => ("", ""),
    };

    let rows = split_rows(content);
    if rows.is_empty() {
        return String::new();
    }

    let mut rendered_rows = Vec::new();
    let mut col_widths: Vec<usize> = Vec::new();

    for row in rows {
        let cols: Vec<String> = row
            .split('&')
            .map(|col| render_math(col, MathMode::Inline))
            .collect();
        if col_widths.len() < cols.len() {
            col_widths.resize(cols.len(), 0);
        }
        for (idx, col) in cols.iter().enumerate() {
            col_widths[idx] = col_widths[idx].max(display_width(col));
        }
        rendered_rows.push(cols);
    }

    let mut lines = Vec::new();
    let col_count = col_widths.len();
    for cols in rendered_rows {
        let mut line = String::new();
        if !left.is_empty() {
            line.push_str(left);
            line.push(' ');
        }
        for idx in 0..col_count {
            let col = cols.get(idx).map(String::as_str).unwrap_or("");
            let padding = col_widths
                .get(idx)
                .copied()
                .unwrap_or(0)
                .saturating_sub(display_width(col));
            line.push_str(col);
            if idx + 1 < col_count {
                line.push_str(&" ".repeat(padding + 2));
            }
        }
        if !right.is_empty() {
            line.push(' ');
            line.push_str(right);
        }
        lines.push(line.trim_end().to_string());
    }

    lines.join("\n")
}

pub(super) fn split_rows(content: &str) -> Vec<String> {
    content
        .split("\\\\")
        .map(|row| row.trim().to_string())
        .filter(|row| !row.is_empty())
        .collect()
}
