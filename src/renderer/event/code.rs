use super::{
    CapturedReferenceBlock, CodeBlockStyle, CodeWrapIndent, CowStr, DeferredLinkReferenceBlock,
    EventRenderer, HighlightLines, LinkStyle, MarkdownProcessor, MdvError, PRETTY_ACCENT_COLOR,
    Result, ThemeElement, WrapMode, as_terminal_escaped, create_style, detect_source_code,
};
use crate::block_spacing::BlockElement;
use crate::inline_style::InlineStyleKind;
use crate::math::is_math_language_hint;
use crate::terminal::AnsiStyle;
use crate::utils::{display_width, strip_ansi};
use regex::regex;
use syntect::parsing::SyntaxReference;
use syntect::util::LinesWithEndings;

const LANGUAGE_SEPARATORS: &[char] = &[' ', '\t', ',', ';', '|'];
const BASIC_CODE_BLOCK_INDENT: usize = 2;

const CUSTOM_LANGUAGE_LABELS: &[(&str, &str)] = &[
    ("bash", "Bash"),
    ("shell", "Shell"),
    ("shell-session", "Shell"),
    ("console", "Shell"),
    ("sh", "Shell"),
    ("objective-c", "Objective-C"),
    ("Javascript (Babel)", "JavaScript"),
];

#[derive(Debug, Clone)]
struct WrappedCodeSegment {
    text: String,
    visible_width: usize,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct CodeBlockRenderInput<'a> {
    highlighted: &'a str,
    language_label: Option<&'a str>,
    code_starts_with_blank: bool,
    should_wrap: bool,
    wrap_mode: WrapMode,
    terminal_width: usize,
    raw_code: &'a str,
}

impl<'a> CodeBlockRenderInput<'a> {
    pub(super) fn new(
        highlighted: &'a str,
        language_label: Option<&'a str>,
        code_starts_with_blank: bool,
        should_wrap: bool,
        wrap_mode: WrapMode,
        terminal_width: usize,
        raw_code: &'a str,
    ) -> Self {
        Self {
            highlighted,
            language_label,
            code_starts_with_blank,
            should_wrap,
            wrap_mode,
            terminal_width,
            raw_code,
        }
    }
}

mod aliases;
mod block;
mod highlighting;
mod hint;
mod inline;
mod labels;
mod plaintext;
mod pretty;
mod rendering;
mod syntax;

struct PlaintextRenderResult {
    body: String,
    references: Vec<CapturedReferenceBlock>,
    deferred_references: Vec<DeferredLinkReferenceBlock>,
    document_links: Vec<(String, String)>,
    reference_counter: usize,
}

#[cfg(test)]
mod tests;
