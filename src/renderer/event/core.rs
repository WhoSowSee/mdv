use super::{
    Alignment, CalloutStyle, Config, Event, FootnoteDefinition, FootnoteStyle, HashMap,
    HeadingLevel, LinkStyle, Result, SyntaxSet, Tag, TagEnd, Theme, ThemeElement, create_style,
    extract_code_language,
};
use crate::theme::Color;
use crate::utils::strip_ansi;
use pulldown_cmark::BlockQuoteKind;
use std::collections::VecDeque;
use syntect::highlighting::Theme as SyntectTheme;

#[derive(Debug)]
pub(crate) struct ListState {
    pub(super) is_ordered: bool,
    pub(super) counter: usize,
    pub(super) current_item_start: Option<usize>,
    pub(super) current_item_marker_end: Option<usize>,
}

#[derive(Debug)]
pub(crate) struct TableState {
    pub(super) alignments: Vec<Alignment>,
    pub(super) headers: Vec<String>,
    pub(super) rows: Vec<Vec<String>>,
    pub(super) in_header: bool,
    pub(super) current_row: Vec<String>,
    pub(super) current_cell: String,
    pub(super) inline_references: Vec<(String, String)>,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CalloutFold {
    Expanded,
    Collapsed,
}

#[derive(Debug, Clone)]
pub(crate) struct CalloutInfo {
    pub(super) kind: CalloutKind,
    pub(super) label: String,
    pub(super) label_override: Option<String>,
    pub(super) fold: Option<CalloutFold>,
    pub(super) header_rendered: bool,
    pub(super) min_heading_indent: Option<usize>,
    pub(super) inline_link_counter: usize,
    pub(super) inline_links: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub(crate) enum CalloutState {
    Pending,
    Active(CalloutInfo),
    None,
}

#[derive(Debug, Clone)]
pub(crate) struct CapturedReferenceBlock {
    pub(super) lines: Vec<String>,
    pub(super) add_trailing_newline: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FootnoteTextState {
    Idle,
    SawOpenBracket,
    Collecting,
}

fn blockquote_kind_info(kind: BlockQuoteKind) -> (CalloutKind, String) {
    match kind {
        BlockQuoteKind::Note => (CalloutKind::Note, "note".to_string()),
        BlockQuoteKind::Tip => (CalloutKind::Tip, "tip".to_string()),
        BlockQuoteKind::Important => (CalloutKind::Tip, "important".to_string()),
        BlockQuoteKind::Warning => (CalloutKind::Warning, "warning".to_string()),
        BlockQuoteKind::Caution => (CalloutKind::Warning, "caution".to_string()),
    }
}

fn build_callout_palette(theme: &Theme) -> HashMap<CalloutKind, Color> {
    let base = build_base_callout_palette(theme);
    remap_callout_palette(theme, &base)
}

fn build_base_callout_palette(theme: &Theme) -> HashMap<CalloutKind, Color> {
    let mut palette = HashMap::new();
    let mut used: Vec<Color> = Vec::new();
    let fallback = collect_callout_fallback_colors(theme);

    let assignments = [
        (CalloutKind::Note, &theme.link),
        (CalloutKind::Abstract, &theme.table_header),
        (CalloutKind::Info, &theme.h4),
        (CalloutKind::Todo, &theme.emphasis),
        (CalloutKind::Tip, &theme.list_marker),
        (CalloutKind::Success, &theme.h2),
        (CalloutKind::Question, &theme.h5),
        (CalloutKind::Warning, &theme.warning),
        (CalloutKind::Failure, &theme.h1),
        (CalloutKind::Danger, &theme.error),
        (CalloutKind::Bug, &theme.h6),
        (CalloutKind::Example, &theme.code),
        (CalloutKind::Quote, &theme.quote),
    ];

    for (kind, primary) in assignments {
        let color = select_unique_callout_color(primary, &mut used, &fallback);
        palette.insert(kind, color);
    }

    palette
}

fn remap_callout_palette(
    theme: &Theme,
    base: &HashMap<CalloutKind, Color>,
) -> HashMap<CalloutKind, Color> {
    let note = base_callout_color(theme, base, CalloutKind::Note);
    let abstract_color = base_callout_color(theme, base, CalloutKind::Abstract);
    let info = base_callout_color(theme, base, CalloutKind::Info);
    let todo = base_callout_color(theme, base, CalloutKind::Todo);
    let tip = base_callout_color(theme, base, CalloutKind::Tip);
    let success = base_callout_color(theme, base, CalloutKind::Success);
    let question = base_callout_color(theme, base, CalloutKind::Question);
    let danger = base_callout_color(theme, base, CalloutKind::Danger);
    let quote = base_callout_color(theme, base, CalloutKind::Quote);

    let mut palette = HashMap::new();
    palette.insert(CalloutKind::Note, note.clone());
    palette.insert(CalloutKind::Info, note.clone());

    palette.insert(CalloutKind::Abstract, abstract_color.clone());
    palette.insert(CalloutKind::Example, abstract_color.clone());

    palette.insert(CalloutKind::Todo, todo);
    palette.insert(CalloutKind::Tip, tip);

    palette.insert(CalloutKind::Success, quote);
    palette.insert(CalloutKind::Warning, success);

    palette.insert(CalloutKind::Question, info);

    palette.insert(CalloutKind::Failure, question.clone());
    palette.insert(CalloutKind::Danger, question.clone());
    palette.insert(CalloutKind::Bug, question);

    palette.insert(CalloutKind::Quote, danger);

    palette
}

fn base_callout_color(
    theme: &Theme,
    base: &HashMap<CalloutKind, Color>,
    kind: CalloutKind,
) -> Color {
    base.get(&kind)
        .cloned()
        .unwrap_or_else(|| theme.text.clone())
}

fn select_unique_callout_color(
    primary: &Color,
    used: &mut Vec<Color>,
    fallback: &[Color],
) -> Color {
    if !used.contains(primary) {
        used.push(primary.clone());
        return primary.clone();
    }

    if let Some(color) = fallback.iter().find(|candidate| !used.contains(candidate)) {
        used.push(color.clone());
        return color.clone();
    }

    primary.clone()
}

fn collect_callout_fallback_colors(theme: &Theme) -> Vec<Color> {
    let candidates = [
        theme.h1.clone(),
        theme.h2.clone(),
        theme.h3.clone(),
        theme.h4.clone(),
        theme.h5.clone(),
        theme.h6.clone(),
        theme.link.clone(),
        theme.emphasis.clone(),
        theme.strong.clone(),
        theme.list_marker.clone(),
        theme.table_header.clone(),
        theme.table_border.clone(),
        theme.error.clone(),
        theme.warning.clone(),
        theme.quote.clone(),
        theme.code.clone(),
        theme.text.clone(),
        theme.text_light.clone(),
        theme.border.clone(),
        theme.syntax.keyword.clone(),
        theme.syntax.string.clone(),
        theme.syntax.comment.clone(),
        theme.syntax.number.clone(),
        theme.syntax.operator.clone(),
        theme.syntax.function.clone(),
        theme.syntax.variable.clone(),
        theme.syntax.type_name.clone(),
        Color::AnsiValue(33),
        Color::AnsiValue(39),
        Color::AnsiValue(45),
        Color::AnsiValue(51),
        Color::AnsiValue(75),
        Color::AnsiValue(81),
        Color::AnsiValue(99),
        Color::AnsiValue(111),
        Color::AnsiValue(135),
        Color::AnsiValue(141),
        Color::AnsiValue(171),
        Color::AnsiValue(203),
        Color::AnsiValue(215),
        Color::AnsiValue(221),
        Color::AnsiValue(227),
    ];

    let mut unique = Vec::new();
    for color in candidates {
        if !unique.contains(&color) {
            unique.push(color);
        }
    }

    unique
}

/// Internal event renderer
pub(crate) struct EventRenderer<'a> {
    pub(crate) config: &'a Config,
    pub(crate) theme: &'a Theme,
    pub(crate) syntax_set: &'a SyntaxSet,
    pub(crate) code_theme: &'a SyntectTheme,
    pub(crate) output: String,
    pub(crate) current_indent: usize,
    pub(crate) blockquote_level: usize,
    pub(crate) blockquote_starts: Vec<usize>,
    pub(crate) callout_stack: Vec<CalloutState>,
    pub(crate) callout_palette: HashMap<CalloutKind, Color>,
    pub(crate) list_stack: Vec<ListState>,
    pub(crate) table_state: Option<TableState>,
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
    pub(crate) plaintext_code_block_depth: usize,
    pub(crate) captured_reference_blocks: Vec<CapturedReferenceBlock>,
    pub(crate) footnote_definitions: Vec<FootnoteDefinition>,
    pub(crate) footnote_order: Vec<String>,
    pub(crate) current_inline_footnotes: Vec<String>,
    pub(crate) footnote_use_count: HashMap<String, usize>,
    pub(crate) suppress_footnote_output: bool,
    pub(crate) footnote_text_state: FootnoteTextState,
    pub(crate) footnote_text_buffer: String,
    pub(crate) last_header_level: HeadingLevel,
    pub(crate) formatting_stack: Vec<ThemeElement>,
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

impl<'a> EventRenderer<'a> {
    pub(crate) fn new(
        config: &'a Config,
        theme: &'a Theme,
        syntax_set: &'a SyntaxSet,
        code_theme: &'a SyntectTheme,
    ) -> Self {
        Self {
            config,
            theme,
            syntax_set,
            code_theme,
            output: String::new(),
            current_indent: 0,
            blockquote_level: 0,
            blockquote_starts: Vec::new(),
            callout_stack: Vec::new(),
            callout_palette: build_callout_palette(theme),
            list_stack: Vec::new(),
            table_state: None,
            link_references: HashMap::new(),
            link_counter: 0,
            current_link_text: String::new(),
            in_link: false,
            paragraph_link_counter: 0,
            paragraph_links: Vec::new(),
            document_links: Vec::new(),
            in_code_block: false,
            code_block_content: String::new(),
            code_block_language: None,
            plaintext_code_block_depth: 0,
            captured_reference_blocks: Vec::new(),
            footnote_definitions: Vec::new(),
            footnote_order: Vec::new(),
            current_inline_footnotes: Vec::new(),
            footnote_use_count: HashMap::new(),
            suppress_footnote_output: false,
            footnote_text_state: FootnoteTextState::Idle,
            footnote_text_buffer: String::new(),
            last_header_level: HeadingLevel::H1,
            formatting_stack: Vec::new(),
            current_heading_level: None,
            current_heading_start: None,
            pending_heading_placeholder: None,
            heading_indent: 0,
            content_indent: 0,
            blockquote_indent_stack: Vec::new(),
            smart_level_indents: HashMap::new(),
            prepared_blockquote_smart_indents: VecDeque::new(),
            active_blockquote_smart_indents: Vec::new(),
            current_paragraph_start: None,
            current_paragraph_has_content: false,
            current_paragraph_has_leading_break: false,
            explicit_blank_line_streak: 0,
            pending_task_marker: false,
            pending_task_marker_buffer: String::new(),
            pending_callout_marker: false,
            pending_callout_marker_buffer: String::new(),
            pending_callout_label_override: false,
            pending_callout_label_buffer: String::new(),
            suppress_next_soft_break: false,
            suppress_next_paragraph_break: false,
        }
    }

    pub(crate) fn render_events(&mut self, events: Vec<Event<'static>>) -> Result<String> {
        let (events, mut definitions) = self.extract_footnote_definitions(events);

        if !self.footnote_definitions.is_empty() {
            for existing in self.footnote_definitions.iter() {
                if !definitions.iter().any(|def| def.name == existing.name) {
                    definitions.push(existing.clone());
                }
            }
        }
        self.footnote_definitions = definitions;

        if matches!(self.config.heading_layout, crate::cli::HeadingLayout::Level)
            && self.config.smart_indent
        {
            self.prepare_smart_heading_indents(&events);
        } else {
            self.smart_level_indents.clear();
        }

        for event in events {
            self.process_event(event)?;
        }

        self.finalize_pending_heading_placeholder();
        if matches!(self.config.footnote_style, FootnoteStyle::Attached)
            && !self.current_inline_footnotes.is_empty()
        {
            self.finalize_inline_footnotes(true, false)?;
        }
        self.finalize_document_link_references();
        self.finalize_document_footnotes()?;

        // Remove excessive trailing newlines, but keep one
        let mut result = self.output.trim_end().to_string();
        if !result.is_empty() {
            result.push('\n');
        }

        Ok(result)
    }

    fn prepare_smart_heading_indents(&mut self, events: &[Event]) {
        self.smart_level_indents.clear();
        self.prepared_blockquote_smart_indents.clear();

        let mut present = [false; 6];
        struct BlockquoteScanFrame {
            start_index: usize,
            present: [bool; 6],
        }

        let mut blockquote_stack: Vec<BlockquoteScanFrame> = Vec::new();
        let mut blockquote_maps: Vec<Option<HashMap<HeadingLevel, usize>>> =
            vec![None; events.len()];

        for (idx, event) in events.iter().enumerate() {
            match event {
                Event::Start(Tag::BlockQuote(_)) => {
                    blockquote_stack.push(BlockquoteScanFrame {
                        start_index: idx,
                        present: [false; 6],
                    });
                }
                Event::End(TagEnd::BlockQuote(_)) => {
                    if let Some(frame) = blockquote_stack.pop() {
                        let map = Self::build_smart_indent_map(&frame.present);
                        blockquote_maps[frame.start_index] = Some(map);
                    }
                }
                Event::Start(Tag::Heading { level, .. }) => {
                    let idx = Self::heading_level_to_number(*level) - 1;
                    if blockquote_stack.is_empty() {
                        present[idx] = true;
                    } else {
                        for frame in blockquote_stack.iter_mut() {
                            frame.present[idx] = true;
                        }
                    }
                }
                _ => {}
            }
        }

        self.smart_level_indents = Self::build_smart_indent_map(&present);
        for entry in blockquote_maps.into_iter().flatten() {
            self.prepared_blockquote_smart_indents.push_back(entry);
        }
    }

    fn build_smart_indent_map(present: &[bool; 6]) -> HashMap<HeadingLevel, usize> {
        let mut map = HashMap::new();
        let min_idx = match present.iter().position(|&is_present| is_present) {
            Some(idx) => idx,
            None => return map,
        };

        for (idx, is_present) in present.iter().enumerate() {
            if !is_present {
                continue;
            }

            let missing_between = (min_idx + 1..idx)
                .filter(|&gap_idx| !present[gap_idx])
                .count();

            let planned_indent = idx.saturating_sub(missing_between).saturating_sub(min_idx);

            if let Some(level) = Self::number_to_heading_level(idx + 1) {
                map.insert(level, planned_indent);
            }
        }

        map
    }

    fn heading_level_to_number(level: HeadingLevel) -> usize {
        match level {
            HeadingLevel::H1 => 1,
            HeadingLevel::H2 => 2,
            HeadingLevel::H3 => 3,
            HeadingLevel::H4 => 4,
            HeadingLevel::H5 => 5,
            HeadingLevel::H6 => 6,
        }
    }

    fn number_to_heading_level(number: usize) -> Option<HeadingLevel> {
        match number {
            1 => Some(HeadingLevel::H1),
            2 => Some(HeadingLevel::H2),
            3 => Some(HeadingLevel::H3),
            4 => Some(HeadingLevel::H4),
            5 => Some(HeadingLevel::H5),
            6 => Some(HeadingLevel::H6),
            _ => None,
        }
    }

    fn process_event(&mut self, event: Event) -> Result<()> {
        if !matches!(event, Event::Text(_)) {
            self.reset_footnote_text_scan();
        }
        match event {
            Event::Start(tag) => self.handle_start_tag(tag)?,
            Event::End(tag_end) => self.handle_end_tag(tag_end)?,
            Event::Text(text) => self.handle_text(text)?,
            Event::Code(code) => self.handle_inline_code(code)?,
            Event::Html(html) => self.handle_html(html)?,
            Event::InlineHtml(html) => self.handle_inline_html(html)?,
            Event::SoftBreak => {
                if self.finalize_pending_callout_label_override() {
                    self.suppress_next_soft_break = true;
                }
                if self.suppress_next_soft_break {
                    self.suppress_next_soft_break = false;
                } else {
                    self.output.push('\n');
                }
            }
            Event::HardBreak => {
                if self.finalize_pending_callout_label_override() {
                    return Ok(());
                }
                if self.current_paragraph_start.is_some() && !self.current_paragraph_has_content {
                    self.current_paragraph_has_leading_break = true;
                    if let Some(start) = self.current_paragraph_start {
                        if start <= self.output.len() {
                            self.output.truncate(start);
                        }
                    }
                } else {
                    self.handle_hard_break();
                }
            }
            Event::Rule => self.handle_horizontal_rule()?,
            Event::FootnoteReference(name) => self.handle_footnote_reference(name)?,
            Event::TaskListMarker(checked) => self.handle_task_list_marker(checked)?,
            Event::InlineMath(math) => self.handle_inline_math(math)?,
            Event::DisplayMath(math) => self.handle_display_math(math)?,
        }
        Ok(())
    }

    fn handle_start_tag(&mut self, tag: Tag) -> Result<()> {
        self.maybe_render_callout_header();
        match tag {
            Tag::Paragraph => {
                self.current_paragraph_start = Some(self.output.len());
                self.current_paragraph_has_content = false;
                self.current_paragraph_has_leading_break = false;

                if matches!(self.config.link_style, LinkStyle::InlineTable) {
                    self.paragraph_link_counter = 0;
                    self.paragraph_links.clear();
                }

                if self.list_stack.is_empty()
                    && !self.output.is_empty()
                    && !self.output.ends_with('\n')
                {
                    self.output.push('\n');
                }

                if self.content_indent > 0
                    && self.table_state.is_none()
                    && self.list_stack.is_empty()
                    && self.blockquote_level == 0
                {
                    if self.output.ends_with('\n') || self.output.is_empty() {
                        self.output.push_str(&" ".repeat(self.content_indent));
                    }
                }
            }
            Tag::Heading { level, .. } => {
                self.handle_header_start(level)?;
            }
            Tag::BlockQuote(kind) => {
                self.blockquote_indent_stack
                    .push((self.content_indent, self.heading_indent));
                self.blockquote_starts.push(self.output.len());
                let smart_map = self
                    .prepared_blockquote_smart_indents
                    .pop_front()
                    .unwrap_or_default();
                self.active_blockquote_smart_indents.push(smart_map);
                self.blockquote_level += 1;
                self.current_indent += 2;
                if !self.output.is_empty() && !self.output.ends_with('\n') {
                    self.output.push('\n');
                }
                let callout_state = match kind {
                    Some(kind) => {
                        let (callout_kind, label) = blockquote_kind_info(kind);
                        CalloutState::Active(CalloutInfo {
                            kind: callout_kind,
                            label,
                            label_override: None,
                            fold: None,
                            header_rendered: false,
                            min_heading_indent: None,
                            inline_link_counter: 0,
                            inline_links: Vec::new(),
                        })
                    }
                    None => CalloutState::Pending,
                };
                self.callout_stack.push(callout_state);
                if matches!(self.config.callout_style.style, CalloutStyle::Pretty)
                    && matches!(self.callout_stack.last(), Some(CalloutState::Active(_)))
                {
                    self.content_indent = 0;
                    self.heading_indent = 0;
                }
            }
            Tag::CodeBlock(kind) => {
                self.in_code_block = true;
                self.code_block_content.clear();
                self.code_block_language = extract_code_language(&kind);
            }
            Tag::List(start_number) => {
                if matches!(self.config.link_style, LinkStyle::InlineTable) {
                    self.paragraph_link_counter = 0;
                    self.paragraph_links.clear();
                }

                let is_ordered = start_number.is_some();
                let counter = start_number.unwrap_or(1) as usize;
                self.list_stack.push(ListState {
                    is_ordered,
                    counter,
                    current_item_start: None,
                    current_item_marker_end: None,
                });
                if !self.output.ends_with('\n') {
                    self.output.push('\n');
                }
            }
            Tag::Item => {
                if self.list_stack.is_empty() {
                    return Ok(());
                }

                self.reset_explicit_blank_line_streak();

                let indent_level = self.list_stack.len().saturating_sub(1);
                let marker = if let Some(list_state) = self.list_stack.last() {
                    if list_state.is_ordered {
                        format!("{}. ", list_state.counter)
                    } else {
                        "- ".to_string()
                    }
                } else {
                    String::new()
                };

                let style = create_style(self.theme, ThemeElement::ListMarker);
                let styled_marker = style.apply(&marker, self.config.no_colors);
                let at_line_start = self.output.ends_with('\n') || self.output.is_empty();

                let start_index = self.output.len();

                if self.blockquote_level > 0 {
                    if at_line_start {
                        let base_indent = if self.current_heading_start.is_some() {
                            self.heading_indent
                        } else {
                            self.content_indent
                        };
                        let indent_after_prefix =
                            self.should_indent_after_blockquote_prefix(self.blockquote_level);
                        if base_indent > 0 && !indent_after_prefix {
                            self.output.push_str(&" ".repeat(base_indent));
                        }
                        let prefix = self.render_blockquote_prefix();
                        self.output.push_str(&prefix);
                        if base_indent > 0 && indent_after_prefix {
                            self.output.push_str(&" ".repeat(base_indent));
                        }
                    }
                } else if self.content_indent > 0 {
                    self.output.push_str(&" ".repeat(self.content_indent));
                }

                let indent = "  ".repeat(indent_level);
                self.output.push_str(&indent);
                self.output.push_str(&styled_marker);

                let marker_end = self.output.len();
                self.commit_pending_heading_placeholder_if_content();

                if let Some(list_state) = self.list_stack.last_mut() {
                    list_state.current_item_start = Some(start_index);
                    list_state.current_item_marker_end = Some(marker_end);

                    if list_state.is_ordered {
                        list_state.counter += 1;
                    }
                }
                self.pending_task_marker = true;
                self.pending_task_marker_buffer.clear();
            }
            Tag::Table(alignments) => {
                if matches!(self.config.link_style, LinkStyle::InlineTable) {
                    self.paragraph_link_counter = 0;
                    self.paragraph_links.clear();
                }

                self.table_state = Some(TableState {
                    alignments,
                    headers: Vec::new(),
                    rows: Vec::new(),
                    in_header: true,
                    current_row: Vec::new(),
                    current_cell: String::new(),
                    inline_references: Vec::new(),
                });
            }
            Tag::TableHead => {
                if let Some(ref mut table) = self.table_state {
                    table.in_header = true;
                }
            }
            Tag::TableRow => {
                if let Some(ref mut table) = self.table_state {
                    table.current_row.clear();
                }
            }
            Tag::TableCell => {
                if let Some(ref mut table) = self.table_state {
                    table.current_cell.clear();
                }
            }
            Tag::Emphasis => {
                self.formatting_stack.push(ThemeElement::Emphasis);
            }
            Tag::Strong => {
                self.formatting_stack.push(ThemeElement::Strong);
            }
            Tag::Strikethrough => {
                self.formatting_stack.push(ThemeElement::Strikethrough);
            }
            Tag::Link { dest_url, .. } => {
                self.handle_link_start(dest_url)?;
            }
            Tag::Image { dest_url, .. } => {
                self.handle_image_start(dest_url)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_end_tag(&mut self, tag_end: TagEnd) -> Result<()> {
        match tag_end {
            TagEnd::Paragraph => {
                self.finalize_pending_callout_label_override();
                let paragraph_start = self.current_paragraph_start.take();
                let paragraph_has_content = self.current_paragraph_has_content;
                let paragraph_has_leading_break = self.current_paragraph_has_leading_break;
                self.current_paragraph_has_content = false;
                self.current_paragraph_has_leading_break = false;
                self.suppress_next_soft_break = false;
                let suppress_break = self.suppress_next_paragraph_break;
                self.suppress_next_paragraph_break = false;

                if matches!(self.config.link_style, LinkStyle::InlineTable)
                    && !self.paragraph_links.is_empty()
                {
                    self.add_paragraph_link_references();
                }

                let inline_footnotes_rendered =
                    matches!(self.config.footnote_style, FootnoteStyle::Attached)
                        && self.has_renderable_footnotes(&self.current_inline_footnotes)
                        && !self.suppress_footnote_output;

                self.finalize_inline_footnotes(true, !self.list_stack.is_empty())?;

                let has_visible_content = paragraph_has_content
                    || paragraph_start.map_or(false, |start| {
                        let slice = if start <= self.output.len() {
                            &self.output[start..]
                        } else {
                            ""
                        };
                        let clean = strip_ansi(slice);
                        clean
                            .chars()
                            .any(|ch| !ch.is_whitespace() && ch != '│' && ch != '┃')
                    });

                if paragraph_has_leading_break && !has_visible_content {
                    self.trim_trailing_blank_lines();
                    self.ensure_contextual_blank_line();
                    return Ok(());
                }

                if !has_visible_content && self.blockquote_level > 0 {
                    self.ensure_contextual_blank_line();
                    return Ok(());
                }

                let skip_blank_line = self.blockquote_level > 0
                    && self.trailing_blank_line_matches(&self.current_line_prefix());

                if self.list_stack.is_empty()
                    && !inline_footnotes_rendered
                    && !suppress_break
                    && !skip_blank_line
                {
                    self.output.push('\n');
                }
            }
            TagEnd::Heading(level) => {
                self.handle_header_end(level)?;
            }
            TagEnd::BlockQuote(_) => {
                let callout_info = match self.callout_stack.last() {
                    Some(CalloutState::Active(info)) => Some(info.clone()),
                    _ => None,
                };
                let was_callout = callout_info.is_some();
                let callout_inline_links = callout_info
                    .as_ref()
                    .map(|info| info.inline_links.clone())
                    .unwrap_or_default();
                let callout_level = self.blockquote_level;
                let start_index = self
                    .blockquote_starts
                    .pop()
                    .unwrap_or_else(|| self.output.len());
                let slice = if start_index <= self.output.len() {
                    self.output[start_index..].to_string()
                } else {
                    String::new()
                };
                let trimmed = strip_ansi(&slice);

                let mut has_visible_content = !trimmed.trim().is_empty();
                if was_callout {
                    has_visible_content = true;
                }

                let use_pretty_callout =
                    was_callout && matches!(self.config.callout_style.style, CalloutStyle::Pretty);

                if use_pretty_callout {
                    if start_index <= self.output.len() {
                        self.output.truncate(start_index);
                    }

                    self.callout_stack.pop();
                    self.pending_callout_marker = false;
                    self.pending_callout_marker_buffer.clear();
                    self.pending_callout_label_override = false;
                    self.pending_callout_label_buffer.clear();
                    self.suppress_next_soft_break = false;
                    self.blockquote_level = self.blockquote_level.saturating_sub(1);
                    self.current_indent = self.current_indent.saturating_sub(2);
                    if let Some((content_indent, heading_indent)) =
                        self.blockquote_indent_stack.pop()
                    {
                        self.content_indent = content_indent;
                        self.heading_indent = heading_indent;
                    }
                    self.active_blockquote_smart_indents.pop();

                    if has_visible_content || self.config.show_empty_elements {
                        if let Some(info) = callout_info {
                            let rendered = self.render_callout_pretty_block(
                                &slice,
                                callout_level,
                                info.kind,
                                &info.label,
                                info.label_override.as_deref(),
                                info.fold,
                            );

                            if !rendered {
                                self.output.push_str(&slice);
                                if !self.output.ends_with('\n') {
                                    self.output.push('\n');
                                }
                                self.ensure_contextual_blank_line();
                            }
                        }
                    }

                    if matches!(self.config.link_style, LinkStyle::InlineTable)
                        && !callout_inline_links.is_empty()
                    {
                        self.trim_trailing_blank_lines();
                        let in_list = !self.list_stack.is_empty();
                        self.render_link_reference_blocks(
                            &callout_inline_links,
                            true,
                            in_list,
                            false,
                        );
                    }

                    return Ok(());
                }

                if !has_visible_content {
                    if self.config.show_empty_elements {
                        if start_index <= self.output.len() {
                            self.output.truncate(start_index);
                        }
                        if !self.output.ends_with('\n') && !self.output.is_empty() {
                            self.output.push('\n');
                        }
                        self.push_indent_for_line_start();
                        if !self.output.ends_with('\n') {
                            self.output.push('\n');
                        }
                    } else if start_index <= self.output.len() {
                        self.output.truncate(start_index);
                    }
                } else if !self.output.ends_with('\n') {
                    self.output.push('\n');
                }

                self.callout_stack.pop();
                self.pending_callout_marker = false;
                self.pending_callout_marker_buffer.clear();
                self.pending_callout_label_override = false;
                self.pending_callout_label_buffer.clear();
                self.suppress_next_soft_break = false;
                self.blockquote_level = self.blockquote_level.saturating_sub(1);
                self.current_indent = self.current_indent.saturating_sub(2);
                if let Some((content_indent, heading_indent)) = self.blockquote_indent_stack.pop() {
                    self.content_indent = content_indent;
                    self.heading_indent = heading_indent;
                }
                self.active_blockquote_smart_indents.pop();

                if was_callout && (has_visible_content || self.config.show_empty_elements) {
                    self.ensure_contextual_blank_line();
                }

                if matches!(self.config.link_style, LinkStyle::InlineTable)
                    && !callout_inline_links.is_empty()
                {
                    self.trim_trailing_blank_lines();
                    let in_list = !self.list_stack.is_empty();
                    self.render_link_reference_blocks(&callout_inline_links, true, in_list, false);
                }
            }
            TagEnd::CodeBlock => {
                self.handle_code_block_end()?;
                if matches!(self.config.footnote_style, FootnoteStyle::Attached)
                    && !self.current_inline_footnotes.is_empty()
                {
                    self.finalize_inline_footnotes(true, false)?;
                }
            }
            TagEnd::List(_) => {
                self.list_stack.pop();

                if matches!(self.config.link_style, LinkStyle::InlineTable)
                    && !self.paragraph_links.is_empty()
                {
                    self.add_paragraph_link_references();
                } else if !self.output.ends_with('\n') {
                    self.output.push('\n');
                }
            }
            TagEnd::Item => {
                let mut start_index = self.output.len();
                let mut has_content = false;
                let mut was_ordered = false;

                if let Some(list_state) = self.list_stack.last_mut() {
                    start_index = list_state
                        .current_item_start
                        .unwrap_or_else(|| self.output.len())
                        .min(self.output.len());
                    let marker_end = list_state
                        .current_item_marker_end
                        .unwrap_or(start_index)
                        .min(self.output.len());
                    let slice = &self.output[marker_end..];
                    has_content = !strip_ansi(slice).trim().is_empty();
                    was_ordered = list_state.is_ordered;
                }

                if matches!(self.config.footnote_style, FootnoteStyle::Attached)
                    && !self.current_inline_footnotes.is_empty()
                {
                    self.finalize_inline_footnotes(true, true)?;
                }

                if let Some(list_state) = self.list_stack.last_mut() {
                    if has_content || self.config.show_empty_elements {
                        if !self.output.ends_with('\n') {
                            self.output.push('\n');
                        }
                    } else {
                        self.output.truncate(start_index);
                        if was_ordered {
                            list_state.counter = list_state.counter.saturating_sub(1);
                        }
                    }

                    list_state.current_item_start = None;
                    list_state.current_item_marker_end = None;
                } else if !self.output.ends_with('\n') {
                    self.output.push('\n');
                }
                self.pending_task_marker = false;
                self.pending_task_marker_buffer.clear();
            }
            TagEnd::Table => {
                self.handle_table_end()?;
                if matches!(self.config.footnote_style, FootnoteStyle::Attached)
                    && !self.current_inline_footnotes.is_empty()
                {
                    self.finalize_inline_footnotes(true, false)?;
                }
            }
            TagEnd::TableHead => {
                if let Some(ref mut table) = self.table_state {
                    table.in_header = false;
                    table.headers = table.current_row.clone();
                }
            }
            TagEnd::TableRow => {
                if let Some(ref mut table) = self.table_state {
                    if !table.in_header {
                        table.rows.push(table.current_row.clone());
                    }
                }
            }
            TagEnd::TableCell => {
                if let Some(ref mut table) = self.table_state {
                    table.current_row.push(table.current_cell.clone());
                }
            }
            TagEnd::Link => {
                self.handle_link_end()?;
            }
            TagEnd::Image => {
                self.handle_image_end()?;
            }
            TagEnd::Emphasis => {
                self.formatting_stack
                    .retain(|&x| x != ThemeElement::Emphasis);
            }
            TagEnd::Strong => {
                self.formatting_stack.retain(|&x| x != ThemeElement::Strong);
            }
            TagEnd::Strikethrough => {
                self.formatting_stack
                    .retain(|&x| x != ThemeElement::Strikethrough);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_hard_break(&mut self) {
        if self.has_trailing_blank_line() {
            return;
        }

        if self.output.ends_with('\n') {
            self.output.push('\n');
        } else {
            self.output.push_str("\n\n");
        }
    }
}
