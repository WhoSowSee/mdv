use super::{
    Alignment, CalloutStyle, Config, DefinitionListState, Event, FootnoteDefinition, FootnoteStyle,
    HashMap, HeadingLevel, LinkStyle, Result, SyntaxSet, Tag, TagEnd, Theme, ThemeElement,
    create_style, extract_code_language,
};
use crate::block_spacing::BlockElement;
use crate::inline_style::InlineStyleKind;
use crate::renderer::syntax_theme::CodeHighlightTheme;
use crate::theme::Color;
use crate::utils::strip_ansi;
use pulldown_cmark::BlockQuoteKind;
use std::collections::VecDeque;

mod callouts;
mod constructor;
mod end_blockquote;
mod end_lists;
mod end_paragraph;
mod end_tags;
mod process;
mod render;
mod start_tags;
mod state;

pub(crate) use state::{
    CalloutFold, CalloutInfo, CalloutKind, CalloutState, CapturedReferenceBlock,
    DeferredLinkReferenceBlock, FootnoteTextState, HtmlBlockBuffer, ListState,
    TableInlineUrlSegment, TableInlineUrlTarget, TableState,
};

use callouts::{blockquote_kind_info, build_callout_palette};

/// Internal event renderer
pub(crate) struct EventRenderer<'a> {
    pub(crate) config: &'a Config,
    pub(crate) theme: &'a Theme,
    pub(crate) syntax_set: &'a SyntaxSet,
    pub(crate) code_theme: &'a CodeHighlightTheme,
    pub(crate) output: String,
    pub(crate) current_indent: usize,
    pub(crate) blockquote_level: usize,
    pub(crate) blockquote_starts: Vec<usize>,
    pub(crate) callout_stack: Vec<CalloutState>,
    pub(crate) callout_palette: HashMap<CalloutKind, Color>,
    pub(crate) list_stack: Vec<ListState>,
    pub(crate) prepared_list_spacing_elements: VecDeque<BlockElement>,
    pub(crate) prepared_blockquote_spacing_elements: VecDeque<BlockElement>,
    pub(super) definition_list_stack: Vec<DefinitionListState>,
    pub(crate) table_state: Option<TableState>,
    pub(crate) pending_html_block_buffer: Option<HtmlBlockBuffer>,
    pub(crate) link_references: HashMap<String, String>,
    pub(crate) link_counter: usize,
    pub(crate) current_link_text: String,
    pub(crate) in_link: bool,
    pub(crate) paragraph_link_counter: usize,
    pub(crate) paragraph_links: Vec<(String, String)>,
    pub(crate) document_links: Vec<(String, String)>,
    pub(crate) in_code_block: bool,
    pub(crate) code_block_content: String,
    pub(crate) code_block_language: Option<String>,
    pub(crate) max_code_line_number_width: usize,
    pub(crate) plaintext_code_block_depth: usize,
    pub(crate) captured_reference_blocks: Vec<CapturedReferenceBlock>,
    pub(crate) deferred_reference_blocks: Vec<DeferredLinkReferenceBlock>,
    pub(crate) footnote_definitions: Vec<FootnoteDefinition>,
    pub(crate) footnote_order: Vec<String>,
    pub(crate) current_inline_footnotes: Vec<String>,
    pub(crate) footnote_use_count: HashMap<String, usize>,
    pub(crate) suppress_footnote_output: bool,
    pub(crate) footnote_text_state: FootnoteTextState,
    pub(crate) footnote_text_buffer: String,
    pub(crate) last_header_level: HeadingLevel,
    pub(crate) formatting_stack: Vec<ThemeElement>,
    pub(crate) active_backtick_style: Option<InlineStyleKind>,
    pub(crate) current_heading_level: Option<HeadingLevel>,
    pub(crate) current_heading_start: Option<usize>,
    pub(crate) pending_heading_placeholder: Option<(usize, usize)>,
    pub(crate) heading_indent: usize,
    pub(crate) content_indent: usize,
    pub(crate) blockquote_indent_stack: Vec<(usize, usize)>,
    pub(crate) smart_level_indents: HashMap<HeadingLevel, usize>,
    pub(crate) prepared_blockquote_smart_indents: VecDeque<HashMap<HeadingLevel, usize>>,
    pub(crate) active_blockquote_smart_indents: Vec<HashMap<HeadingLevel, usize>>,
    pub(crate) current_paragraph_start: Option<usize>,
    pub(crate) current_paragraph_has_content: bool,
    pub(crate) current_paragraph_has_leading_break: bool,
    pub(crate) explicit_blank_line_streak: usize,
    pub(crate) pending_task_marker: bool,
    pub(crate) pending_task_marker_buffer: String,
    pub(crate) pending_callout_marker: bool,
    pub(crate) pending_callout_marker_buffer: String,
    pub(crate) pending_callout_label_override: bool,
    pub(crate) pending_callout_label_buffer: String,
    pub(crate) suppress_next_soft_break: bool,
    pub(crate) suppress_next_paragraph_break: bool,
}
