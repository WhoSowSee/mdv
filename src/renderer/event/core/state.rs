use super::*;

#[derive(Debug)]
pub(crate) struct ListState {
    pub(in crate::renderer::event) is_ordered: bool,
    pub(in crate::renderer::event) counter: usize,
    pub(in crate::renderer::event) block_start: usize,
    pub(in crate::renderer::event) has_visible_items: bool,
    pub(in crate::renderer::event) current_item_start: Option<usize>,
    pub(in crate::renderer::event) current_item_marker_start: Option<usize>,
    pub(in crate::renderer::event) current_item_marker_end: Option<usize>,
    pub(in crate::renderer::event) spacing_element: BlockElement,
}

#[derive(Debug)]
pub(crate) struct TableState {
    pub(in crate::renderer::event) alignments: Vec<Alignment>,
    pub(in crate::renderer::event) headers: Vec<String>,
    pub(in crate::renderer::event) rows: Vec<Vec<String>>,
    pub(in crate::renderer::event) in_header: bool,
    pub(in crate::renderer::event) current_row: Vec<String>,
    pub(in crate::renderer::event) current_cell: String,
    pub(in crate::renderer::event) clickable_link_replacements: Vec<(String, String)>,
    pub(in crate::renderer::event) inline_url_segments: Vec<TableInlineUrlSegment>,
}

#[derive(Debug)]
pub(crate) struct HtmlBlockBuffer {
    pub(in crate::renderer::event) tag: &'static str,
    pub(in crate::renderer::event) content: String,
    pub(in crate::renderer::event) captures_markdown_events: bool,
}

#[derive(Debug, Clone)]
pub(crate) enum TableInlineUrlTarget {
    Header {
        column_index: usize,
    },
    Row {
        row_index: usize,
        column_index: usize,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct TableInlineUrlSegment {
    pub(in crate::renderer::event) target: TableInlineUrlTarget,
    pub(in crate::renderer::event) url: String,
    pub(in crate::renderer::event) url_part: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum CalloutKind {
    Note,
    Abstract,
    Info,
    Todo,
    Tip,
    Success,
    Question,
    Warning,
    Failure,
    Danger,
    Bug,
    Example,
    Quote,
    Properties,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CalloutFold {
    Expanded,
    Collapsed,
}

#[derive(Debug, Clone)]
pub(crate) struct CalloutInfo {
    pub(in crate::renderer::event) kind: CalloutKind,
    pub(in crate::renderer::event) label: String,
    pub(in crate::renderer::event) label_override: Option<String>,
    pub(in crate::renderer::event) fold: Option<CalloutFold>,
    pub(in crate::renderer::event) header_rendered: bool,
    pub(in crate::renderer::event) min_heading_indent: Option<usize>,
    pub(in crate::renderer::event) inline_link_counter: usize,
    pub(in crate::renderer::event) inline_links: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub(crate) enum CalloutState {
    Pending,
    Active(CalloutInfo),
    None,
}

#[derive(Debug, Clone)]
pub(crate) struct CapturedReferenceBlock {
    pub(in crate::renderer::event) lines: Vec<String>,
    pub(in crate::renderer::event) add_trailing_newline: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct DeferredLinkReferenceBlock {
    pub(in crate::renderer::event) links: Vec<(String, String)>,
    pub(in crate::renderer::event) add_trailing_newline: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FootnoteTextState {
    Idle,
    SawOpenBracket,
    Collecting,
}
