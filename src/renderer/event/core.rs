use super::{
    Alignment, Config, Event, HashMap, HeadingLevel, LinkStyle, Result, SyntaxSet, Tag, TagEnd,
    Theme, ThemeElement, create_style, extract_code_language,
};
use crate::utils::strip_ansi;
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

#[derive(Debug, Clone)]
pub(crate) struct CapturedReferenceBlock {
    pub(super) lines: Vec<String>,
    pub(super) add_trailing_newline: bool,
    pub(super) in_list: bool,
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
    pub(crate) list_stack: Vec<ListState>,
    pub(crate) table_state: Option<TableState>,
    pub(crate) link_references: HashMap<String, String>,
    pub(crate) link_counter: usize,
    pub(crate) current_link_text: String,
    pub(crate) in_link: bool,
    pub(crate) paragraph_link_counter: usize,
    pub(crate) paragraph_links: Vec<(String, String)>,
    pub(crate) in_code_block: bool,
    pub(crate) code_block_content: String,
    pub(crate) code_block_language: Option<String>,
    pub(crate) plaintext_code_block_depth: usize,
    pub(crate) captured_reference_blocks: Vec<CapturedReferenceBlock>,
    pub(crate) last_header_level: HeadingLevel,
    pub(crate) formatting_stack: Vec<ThemeElement>,
    pub(crate) current_heading_level: Option<HeadingLevel>,
    pub(crate) current_heading_start: Option<usize>,
    pub(crate) pending_heading_placeholder: Option<(usize, usize)>,
    pub(crate) heading_indent: usize,
    pub(crate) content_indent: usize,
    pub(crate) smart_level_indents: HashMap<HeadingLevel, usize>,
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
            list_stack: Vec::new(),
            table_state: None,
            link_references: HashMap::new(),
            link_counter: 0,
            current_link_text: String::new(),
            in_link: false,
            paragraph_link_counter: 0,
            paragraph_links: Vec::new(),
            in_code_block: false,
            code_block_content: String::new(),
            code_block_language: None,
            plaintext_code_block_depth: 0,
            captured_reference_blocks: Vec::new(),
            last_header_level: HeadingLevel::H1,
            formatting_stack: Vec::new(),
            current_heading_level: None,
            current_heading_start: None,
            pending_heading_placeholder: None,
            heading_indent: 0,
            content_indent: 0,
            smart_level_indents: HashMap::new(),
        }
    }

    pub(crate) fn render_events(&mut self, events: Vec<Event>) -> Result<String> {
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

        // Remove excessive trailing newlines, but keep one
        let mut result = self.output.trim_end().to_string();
        if !result.is_empty() {
            result.push('\n');
        }

        Ok(result)
    }

    fn prepare_smart_heading_indents(&mut self, events: &[Event]) {
        self.smart_level_indents.clear();

        let mut present = [false; 6];
        for event in events {
            if let Event::Start(Tag::Heading { level, .. }) = event {
                let idx = Self::heading_level_to_number(*level) - 1;
                present[idx] = true;
            }
        }

        let min_idx = match present.iter().position(|&is_present| is_present) {
            Some(idx) => idx,
            None => return,
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
                self.smart_level_indents.insert(level, planned_indent);
            }
        }
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
        match event {
            Event::Start(tag) => self.handle_start_tag(tag)?,
            Event::End(tag_end) => self.handle_end_tag(tag_end)?,
            Event::Text(text) => self.handle_text(text)?,
            Event::Code(code) => self.handle_inline_code(code)?,
            Event::Html(html) => self.handle_html(html)?,
            Event::InlineHtml(html) => self.handle_inline_html(html)?,
            Event::SoftBreak => {
                self.output.push('\n');
            }
            Event::HardBreak => self.output.push_str("\n\n"),
            Event::Rule => self.handle_horizontal_rule()?,
            Event::FootnoteReference(name) => self.handle_footnote_reference(name)?,
            Event::TaskListMarker(checked) => self.handle_task_list_marker(checked)?,
            Event::InlineMath(_) | Event::DisplayMath(_) => {
                // Handle math and inline HTML - for now just ignore
            }
        }
        Ok(())
    }

    fn handle_start_tag(&mut self, tag: Tag) -> Result<()> {
        match tag {
            Tag::Paragraph => {
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
            Tag::BlockQuote(_) => {
                self.blockquote_starts.push(self.output.len());
                self.blockquote_level += 1;
                self.current_indent += 2;
                if !self.output.is_empty() && !self.output.ends_with('\n') {
                    self.output.push('\n');
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
                        if self.content_indent > 0 {
                            self.output.push_str(&" ".repeat(self.content_indent));
                        }
                        let prefix = self.render_blockquote_prefix();
                        self.output.push_str(&prefix);
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
                if matches!(self.config.link_style, LinkStyle::InlineTable)
                    && !self.paragraph_links.is_empty()
                {
                    self.add_paragraph_link_references();
                }

                if self.list_stack.is_empty() {
                    self.output.push('\n');
                }
            }
            TagEnd::Heading(level) => {
                self.handle_header_end(level)?;
            }
            TagEnd::BlockQuote(_) => {
                let start_index = self
                    .blockquote_starts
                    .pop()
                    .unwrap_or_else(|| self.output.len());
                let slice = if start_index <= self.output.len() {
                    &self.output[start_index..]
                } else {
                    ""
                };
                let trimmed = strip_ansi(slice);

                if trimmed.trim().is_empty() {
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

                self.blockquote_level = self.blockquote_level.saturating_sub(1);
                self.current_indent = self.current_indent.saturating_sub(2);
            }
            TagEnd::CodeBlock => {
                self.handle_code_block_end()?;
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
                if let Some(list_state) = self.list_stack.last_mut() {
                    let start_index = list_state
                        .current_item_start
                        .unwrap_or_else(|| self.output.len())
                        .min(self.output.len());
                    let marker_end = list_state
                        .current_item_marker_end
                        .unwrap_or(start_index)
                        .min(self.output.len());
                    let slice = &self.output[marker_end..];
                    let has_content = !strip_ansi(slice).trim().is_empty();

                    if has_content || self.config.show_empty_elements {
                        if !self.output.ends_with('\n') {
                            self.output.push('\n');
                        }
                    } else {
                        self.output.truncate(start_index);
                        if list_state.is_ordered {
                            list_state.counter = list_state.counter.saturating_sub(1);
                        }
                    }

                    list_state.current_item_start = None;
                    list_state.current_item_marker_end = None;
                } else if !self.output.ends_with('\n') {
                    self.output.push('\n');
                }
            }
            TagEnd::Table => {
                self.handle_table_end()?;
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
}
