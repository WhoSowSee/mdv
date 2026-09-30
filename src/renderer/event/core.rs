use super::{
    Alignment, CalloutStyle, Config, DefinitionListState, Event, FootnoteDefinition, FootnoteStyle,
    HashMap, HeadingLevel, LinkStyle, Result, SyntaxSet, Tag, TagEnd, Theme, ThemeElement,
    create_style, extract_code_language,
};
use crate::block_spacing::BlockElement;
use crate::inline_style::InlineStyleKind;
use crate::renderer::syntax_theme::CodeHighlightTheme;
use crate::terminal::OutputStyle;
use crate::theme::Color;
use crate::utils::strip_ansi;
use pulldown_cmark::BlockQuoteKind;
use std::collections::VecDeque;
use std::rc::Rc;

mod callouts;
mod constructor;
mod end_blockquote;
mod end_lists;
mod end_paragraph;
mod end_tags;
mod output;
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
pub(in crate::renderer) struct EventRenderer<'a> {
    pub(super) config: &'a Config,
    pub(super) theme: &'a Theme,
    pub(super) syntax_set: &'a SyntaxSet,
    pub(super) code_theme: &'a CodeHighlightTheme,
    pub(super) output_style: OutputStyle,
    pub(super) math_diagnostics: Rc<crate::math::MathDiagnostics>,
    pub(super) output: String,
    pub(super) current_indent: usize,
    pub(super) blockquote_level: usize,
    pub(super) html_details_depth: usize,
    pub(super) pending_html_source_line: Option<usize>,
    pub(super) blockquote_starts: Vec<usize>,
    pub(super) callout_stack: Vec<CalloutState>,
    pub(super) callout_palette: HashMap<CalloutKind, Color>,
    pub(super) list_stack: Vec<ListState>,
    pub(super) prepared_list_spacing_elements: VecDeque<BlockElement>,
    pub(super) prepared_blockquote_spacing_elements: VecDeque<BlockElement>,
    pub(super) definition_list_stack: Vec<DefinitionListState>,
    pub(super) table_state: Option<TableState>,
    pub(super) pending_html_block_buffer: Option<HtmlBlockBuffer>,
    pub(super) link_references: HashMap<String, String>,
    pub(super) link_counter: usize,
    pub(super) current_link_text: String,
    pub(super) in_link: bool,
    pub(super) paragraph_link_counter: usize,
    pub(super) paragraph_links: Vec<(String, String)>,
    pub(super) document_links: Vec<(String, String)>,
    pub(super) in_code_block: bool,
    pub(super) code_block_content: String,
    pub(super) math_code_block_source_line: Option<usize>,
    pub(super) code_block_language: Option<String>,
    pub(super) max_code_line_number_width: usize,
    pub(super) plaintext_code_block_depth: usize,
    pub(super) captured_reference_blocks: Vec<CapturedReferenceBlock>,
    pub(super) deferred_reference_blocks: Vec<DeferredLinkReferenceBlock>,
    pub(super) footnote_definitions: Vec<FootnoteDefinition>,
    pub(super) footnote_order: Vec<String>,
    pub(super) current_inline_footnotes: Vec<String>,
    pub(super) footnote_use_count: HashMap<String, usize>,
    pub(super) suppress_footnote_output: bool,
    pub(super) footnote_text_state: FootnoteTextState,
    pub(super) footnote_text_buffer: String,
    pub(super) last_header_level: HeadingLevel,
    pub(super) formatting_stack: Vec<ThemeElement>,
    pub(super) active_backtick_style: Option<InlineStyleKind>,
    pub(super) current_heading_level: Option<HeadingLevel>,
    pub(super) current_heading_start: Option<usize>,
    pub(super) pending_heading_placeholder: Option<(usize, usize)>,
    pub(super) heading_indent: usize,
    pub(super) content_indent: usize,
    pub(super) blockquote_indent_stack: Vec<(usize, usize)>,
    pub(super) smart_level_indents: HashMap<HeadingLevel, usize>,
    pub(super) prepared_blockquote_smart_indents: VecDeque<HashMap<HeadingLevel, usize>>,
    pub(super) active_blockquote_smart_indents: Vec<HashMap<HeadingLevel, usize>>,
    pub(super) current_paragraph_start: Option<usize>,
    pub(super) current_paragraph_has_content: bool,
    pub(super) current_paragraph_has_leading_break: bool,
    pub(super) explicit_blank_line_streak: usize,
    pub(super) pending_task_marker: bool,
    pub(super) pending_task_marker_buffer: String,
    pub(super) pending_callout_marker: bool,
    pub(super) pending_callout_marker_buffer: String,
    pub(super) pending_callout_label_override: bool,
    pub(super) pending_callout_label_buffer: String,
    pub(super) suppress_next_soft_break: bool,
    pub(super) suppress_next_paragraph_break: bool,
}
