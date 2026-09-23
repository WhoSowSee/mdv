use super::{Metadata, attributes};
use crate::callout::is_valid_callout_name;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Dialect {
    Container,
    Mkdocs,
    Myst,
    Quarto,
    Pymdown,
    Gitbook,
}

#[derive(Clone, Copy)]
pub(super) enum Terminator {
    Fence(char, usize),
    Indent,
    Gitbook,
}

pub(super) struct Opening {
    pub metadata: Metadata,
    pub dialect: Dialect,
    pub terminator: Terminator,
}

impl Opening {
    pub fn parse(line: &str) -> Option<Self> {
        let line = line.trim_end();
        if let Some(rest) = line
            .strip_prefix("{%")
            .and_then(|rest| rest.strip_suffix("%}"))
        {
            let rest = rest.trim().strip_prefix("hint")?;
            if !rest.starts_with(char::is_whitespace) {
                return None;
            }
            let attrs = attributes::parse(rest)?;
            let style = attrs.iter().find(|(key, _)| key == "style")?.1.as_str();
            if !matches!(style, "info" | "success" | "warning" | "danger") {
                return None;
            }
            return Some(Self {
                metadata: Metadata::new(style),
                dialect: Dialect::Gitbook,
                terminator: Terminator::Gitbook,
            });
        }
        let marker = line.chars().next()?;
        if !matches!(marker, ':' | '!' | '?' | '/' | '`' | '~') {
            return None;
        }
        let count = line.chars().take_while(|ch| *ch == marker).count();
        if count < 3 {
            return None;
        }
        let mut rest = line[count..].trim_start();
        let mut fold = None;
        if marker == '?' {
            fold = Some('-');
            if let Some(after) = rest.strip_prefix('+') {
                fold = Some('+');
                rest = after.trim_start();
            }
        }
        let mut dialect = match marker {
            '!' | '?' => Dialect::Mkdocs,
            '/' => Dialect::Pymdown,
            _ => Dialect::Container,
        };
        let mut metadata;
        if rest.starts_with('{') {
            let (inside, after) = attributes::bracketed(rest, '{', '}')?;
            if is_valid_callout_name(inside) {
                if matches!(marker, '`' | '~') && !myst_kind(inside) {
                    return None;
                }
                dialect = Dialect::Myst;
                metadata = Metadata::new(inside);
                metadata.title = nonempty(after.trim());
                version_title(&mut metadata);
            } else {
                if marker != ':' || !after.trim().is_empty() {
                    return None;
                }
                let attrs = attributes::parse(inside)?;
                let kind = attrs.iter().find_map(|(key, value)| {
                    (key == "class")
                        .then(|| value.strip_prefix("callout-"))
                        .flatten()
                })?;
                if !is_valid_callout_name(kind) {
                    return None;
                }
                dialect = Dialect::Quarto;
                metadata = Metadata::new(kind);
                for (key, value) in attrs {
                    metadata.option(&key, &value);
                }
            }
        } else {
            if matches!(marker, '`' | '~') {
                return None;
            }
            let end = rest
                .find(|ch: char| ch.is_whitespace() || matches!(ch, '[' | '{' | '|'))
                .unwrap_or(rest.len());
            let kind = &rest[..end];
            if !is_valid_callout_name(kind) {
                return None;
            }
            metadata = Metadata::new(kind);
            rest = rest[end..].trim_start();
            if dialect == Dialect::Pymdown {
                if !rest.is_empty() {
                    metadata.title = Some(rest.strip_prefix('|')?.trim().to_string());
                }
            } else if dialect == Dialect::Mkdocs {
                if let Some(index) = rest.find('"') {
                    let (title, after) = attributes::quoted(&rest[index..])?;
                    if !after.trim().is_empty() {
                        return None;
                    }
                    metadata.title = Some(title);
                } else if !rest
                    .split_whitespace()
                    .all(|word| matches!(word, "inline" | "end"))
                {
                    metadata.title = nonempty(rest);
                }
            } else {
                if rest.starts_with('[') {
                    let (title, after) = attributes::bracketed(rest, '[', ']')?;
                    metadata.title = Some(title.to_string());
                    rest = after.trim();
                } else if !rest.starts_with('{') {
                    let attrs_start = rest.char_indices().find_map(|(index, ch)| {
                        (ch == '{'
                            && index > 0
                            && rest[..index].ends_with(char::is_whitespace)
                            && attributes::bracketed(&rest[index..], '{', '}')
                                .is_some_and(|(_, after)| after.trim().is_empty()))
                        .then_some(index)
                    });
                    if let Some(index) = attrs_start {
                        metadata.title = nonempty(rest[..index].trim_end());
                        rest = &rest[index..];
                    } else {
                        metadata.title = nonempty(rest);
                        rest = "";
                    }
                }
                if !rest.is_empty() {
                    let (attrs, after) = attributes::bracketed(rest, '{', '}')?;
                    if !after.trim().is_empty() {
                        return None;
                    }
                    for (key, value) in attributes::parse(attrs)? {
                        metadata.option(&key, &value);
                    }
                }
            }
        }
        if metadata.kind == "details" && metadata.fold.is_none() {
            metadata.fold = Some('-');
        }
        if fold.is_some() {
            metadata.fold = fold;
        }
        Some(Self {
            metadata,
            dialect,
            terminator: if matches!(marker, '!' | '?') {
                Terminator::Indent
            } else {
                Terminator::Fence(marker, count)
            },
        })
    }
}

pub(super) fn known_kind(kind: &str) -> bool {
    matches!(
        kind,
        "note"
            | "abstract"
            | "summary"
            | "tldr"
            | "info"
            | "todo"
            | "tip"
            | "hint"
            | "important"
            | "success"
            | "check"
            | "done"
            | "question"
            | "help"
            | "faq"
            | "warning"
            | "caution"
            | "attention"
            | "failure"
            | "fail"
            | "missing"
            | "danger"
            | "error"
            | "bug"
            | "example"
            | "quote"
            | "cite"
            | "seealso"
    )
}

fn myst_kind(kind: &str) -> bool {
    known_kind(kind)
        || matches!(
            kind,
            "admonition" | "versionadded" | "versionchanged" | "deprecated"
        )
}

fn nonempty(text: &str) -> Option<String> {
    (!text.is_empty()).then(|| text.to_string())
}

fn version_title(metadata: &mut Metadata) {
    let prefix = match metadata.kind.as_str() {
        "versionadded" => "Added in version",
        "versionchanged" => "Changed in version",
        "deprecated" => "Deprecated since version",
        _ => return,
    };
    metadata.title = Some(
        format!("{prefix} {}", metadata.title.as_deref().unwrap_or_default())
            .trim_end()
            .to_string(),
    );
}
