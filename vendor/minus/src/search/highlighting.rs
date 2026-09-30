use super::*;

pub(crate) fn highlight_matches_args<'a, 'b>(
    line: &'a str,
    query: &'b Regex,
    accurate: bool,
) -> HighlightMatchesArgs<'a, 'b> {
    let stripped_str = ANSI_REGEX.replace_all(line, "");
    let is_match = query.is_match(&stripped_str);
    HighlightMatchesArgs {
        line,
        query,
        accurate,
        is_match,
    }
}

pub(super) fn highlight_line_matches_ansi(
    line: &str,
    query: &regex::Regex,
    accurate: bool,
) -> String {
    let stripped_str = ANSI_REGEX.replace_all(line, "");

    if !query.is_match(&stripped_str) {
        return line.to_string();
    }

    let mut sum_width = 0;

    // Original ANSI escapes are tracked by offsets in the stripped string.
    let escapes = ANSI_REGEX
        .find_iter(line)
        .map(|escape| {
            let start = escape.start();
            let as_str = escape.as_str();
            let ret = (start - sum_width, as_str);
            sum_width += as_str.len();
            ret
        })
        .collect::<Vec<_>>();

    let matches = query
        .find_iter(&stripped_str)
        .flat_map(|c| [c.start(), c.end()])
        .collect::<Vec<_>>();

    let mut inverted = query
        .replace_all(&stripped_str, |caps: &regex::Captures| {
            format!("{}{}{}", *INVERT, &caps[0], *NORMAL)
        })
        .to_string();

    let mut inserted_escs_len = 0;
    for esc in escapes {
        let match_count = matches.iter().take_while(|m| **m <= esc.0).count();
        let num_invert = match_count / 2;
        let num_normal = match_count - num_invert;

        // Approximate mode moves escapes inside a match behind that match's reset sequence.
        let mut pos = if !accurate && match_count % 2 == 1 {
            // An odd boundary count guarantees the matching end boundary exists.
            matches.get(match_count).unwrap()
                + NORMAL.len()
                + inserted_escs_len
                + (num_invert * INVERT.len())
                + (num_normal * NORMAL.len())
        } else {
            esc.0 + inserted_escs_len + (num_invert * INVERT.len()) + (num_normal * NORMAL.len())
        };

        if match_count % 2 == 1 {
            pos = pos.saturating_sub(1);
        }

        inverted.insert_str(pos, esc.1);

        inserted_escs_len += esc.1.len();
    }

    inverted
}

#[cfg(test)]
pub(crate) fn highlight_line_matches(
    line: &str,
    query: &regex::Regex,
    accurate: bool,
) -> (String, bool) {
    let highlighted = highlight_matches_args(line, query, accurate);
    (highlighted.to_string(), highlighted.is_match)
}

pub(crate) struct HighlightMatchesArgs<'a, 'b> {
    line: &'a str,
    query: &'b Regex,
    accurate: bool,
    is_match: bool,
}

impl fmt::Display for HighlightMatchesArgs<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.is_match {
            return f.write_str(self.line);
        }

        if !ANSI_REGEX.is_match(self.line) {
            let mut last = 0;
            for matched in self.query.find_iter(self.line) {
                f.write_str(&self.line[last..matched.start()])?;
                write!(f, "{}{}{}", *INVERT, matched.as_str(), *NORMAL)?;
                last = matched.end();
            }
            return f.write_str(&self.line[last..]);
        }

        f.write_str(&highlight_line_matches_ansi(
            self.line,
            self.query,
            self.accurate,
        ))
    }
}
