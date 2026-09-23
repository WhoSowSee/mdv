use super::{Dialect, Line, MarkdownProcessor};
use crate::callout::is_valid_callout_name;
use pulldown_cmark::Event;

const OPTIONS_PREFIX: &str = "\u{001d}MDV_CALLOUT_OPTIONS:";

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CalloutOptions {
    pub(crate) hide_title: bool,
    pub(crate) hide_icon: bool,
}

pub(super) fn options_event(options: CalloutOptions) -> Event<'static> {
    Event::InlineHtml(
        format!(
            "{OPTIONS_PREFIX}{}{}",
            u8::from(options.hide_title),
            u8::from(options.hide_icon)
        )
        .into(),
    )
}

pub(crate) fn options_from_event(event: &Event<'_>) -> Option<CalloutOptions> {
    let Event::InlineHtml(text) = event else {
        return None;
    };
    let flags = text.strip_prefix(OPTIONS_PREFIX)?;
    match flags {
        "00" | "01" | "10" | "11" => Some(CalloutOptions {
            hide_title: flags.starts_with('1'),
            hide_icon: flags.ends_with('1'),
        }),
        _ => None,
    }
}

pub(super) struct Metadata {
    pub kind: String,
    pub title: Option<String>,
    pub fold: Option<char>,
    pub options: CalloutOptions,
}

impl Metadata {
    pub fn new(kind: &str) -> Self {
        Self {
            kind: kind.to_ascii_lowercase(),
            title: None,
            fold: None,
            options: CalloutOptions::default(),
        }
    }

    pub fn option(&mut self, key: &str, value: &str) {
        let value = value.trim();
        match key {
            "title" => self.title = Some(value.to_string()),
            "type" if is_valid_callout_name(value) => self.kind = value.to_ascii_lowercase(),
            "collapse" => match value {
                "true" => self.fold = Some('-'),
                "false" => self.fold = Some('+'),
                _ => {}
            },
            "open" => match value.to_ascii_lowercase().as_str() {
                "true" | "" => self.fold = Some('+'),
                "false" => self.fold = Some('-'),
                _ => {}
            },
            "icon" => self.options.hide_icon = value == "false",
            "appearance" if value == "minimal" => self.options.hide_icon = true,
            "class" | "classes" => {
                let classes: Vec<_> = value.split_whitespace().collect();
                if classes.contains(&"dropdown") {
                    self.fold = Some(if classes.contains(&"toggle-shown") {
                        '+'
                    } else {
                        '-'
                    });
                }
                if self.kind == "admonition"
                    && let Some(kind) = classes.iter().find(|kind| super::syntax::known_kind(kind))
                {
                    self.kind = kind.to_string();
                }
            }
            _ => {}
        }
    }

    pub fn consume_options(&mut self, dialect: Dialect, body: &mut Vec<Line>) {
        if matches!(dialect, Dialect::Myst | Dialect::Pymdown) {
            let end = if dialect == Dialect::Pymdown {
                0
            } else {
                body.iter()
                    .position(|line| !line.text.trim().is_empty())
                    .unwrap_or(body.len())
            };
            let mut index = end;
            if dialect == Dialect::Myst
                && body
                    .get(index)
                    .is_some_and(|line| line.text.trim() == "---")
            {
                if let Some(close) = body[index + 1..]
                    .iter()
                    .position(|line| line.text.trim() == "---")
                {
                    let last = index + 1 + close;
                    if self.apply_yaml_options(&body[index + 1..last]) {
                        body.drain(..=last);
                    }
                }
            } else if dialect == Dialect::Pymdown {
                while body.get(index).is_some_and(|line| {
                    line.text.starts_with("    ") || line.text.starts_with('\t')
                }) {
                    index += 1;
                }
                if self.apply_yaml_options(&body[end..index]) {
                    body.drain(..index);
                }
            } else {
                while let Some(line) = body.get(index) {
                    let Some(rest) = line.text.trim().strip_prefix(':') else {
                        break;
                    };
                    let Some((key, value)) = rest.split_once(':') else {
                        break;
                    };
                    if key.is_empty() || key.chars().any(char::is_whitespace) {
                        break;
                    }
                    self.option(key, value);
                    index += 1;
                }
                body.drain(..index);
            }
        }
        if dialect == Dialect::Quarto {
            let index = body.iter().position(|line| !line.text.trim().is_empty());
            if let Some(index) = index
                && MarkdownProcessor::leading_indent_columns(&body[index].text) <= 3
                && let Some(title) = heading(&body[index].text)
            {
                if self.title.is_none() {
                    self.title = Some(title.to_string());
                }
                body.remove(index);
            }
        }
        if dialect == Dialect::Pymdown
            && let Some(title) = self.title.as_deref().and_then(heading)
        {
            self.title = Some(title.to_string());
        }
        if self.title.as_deref() == Some("") && self.fold.is_none() {
            self.options.hide_title = true;
        }
    }

    fn apply_yaml_options(&mut self, lines: &[Line]) -> bool {
        let yaml = lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let Ok(serde_yaml::Value::Mapping(options)) = serde_yaml::from_str(&yaml) else {
            return false;
        };
        for (key, value) in options {
            if let Some(key) = key.as_str() {
                match value {
                    serde_yaml::Value::String(value) => self.option(key, &value),
                    serde_yaml::Value::Bool(value) => {
                        self.option(key, if value { "true" } else { "false" })
                    }
                    _ => {}
                }
            }
        }
        true
    }
}

fn heading(text: &str) -> Option<&str> {
    let trimmed = text.trim_matches([' ', '\t']);
    let count = trimmed.bytes().take_while(|byte| *byte == b'#').count();
    if !(1..=6).contains(&count) {
        return None;
    }
    let rest = &trimmed[count..];
    if !rest.is_empty() && !rest.starts_with([' ', '\t']) {
        return None;
    }
    let content = rest.trim_start_matches([' ', '\t']);
    let before_hashes = content.trim_end_matches('#');
    if before_hashes.is_empty() || before_hashes.ends_with([' ', '\t']) {
        Some(before_hashes.trim_end_matches([' ', '\t']))
    } else {
        Some(content)
    }
}
