use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn handle_start_tag(&mut self, tag: Tag) -> Result<()> {
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

                if self.list_stack.is_empty() {
                    if self.in_definition_description() {
                        if self.output.ends_with('\n') {
                            self.ensure_contextual_blank_line();
                        }
                    } else {
                        if self.blockquote_level == 0 {
                            let spacing =
                                self.config.block_spacing.spacing(BlockElement::Paragraph);
                            self.ensure_contextual_blank_lines(spacing.top);
                        }
                        if !self.output.is_empty() && !self.output.ends_with('\n') {
                            self.output.push('\n');
                        }
                    }
                }

                if self.content_indent > 0
                    && self.table_state.is_none()
                    && self.list_stack.is_empty()
                    && self.blockquote_level == 0
                    && (self.output.ends_with('\n') || self.output.is_empty())
                {
                    self.output.push_str(&" ".repeat(self.content_indent));
                }
            }
            Tag::Heading { level, .. } => {
                self.handle_header_start(level)?;
            }
            Tag::BlockQuote(kind) => {
                let entering_outer_blockquote = self.blockquote_level == 0;
                let blockquote_start = self.output.len();
                let spacing_element = self
                    .prepared_blockquote_spacing_elements
                    .pop_front()
                    .expect("blockquote spacing classification must match the event stream");
                if entering_outer_blockquote {
                    let spacing = self.config.block_spacing.spacing(spacing_element);
                    self.ensure_contextual_blank_lines(spacing.top);
                }
                self.blockquote_indent_stack
                    .push((self.content_indent, self.heading_indent));
                self.blockquote_starts.push(blockquote_start);
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
                let entering_top_level_list = self.list_stack.is_empty();
                let block_start = self.output.len();
                let spacing_element = self
                    .prepared_list_spacing_elements
                    .pop_front()
                    .expect("list spacing classification must match the event stream");
                if entering_top_level_list {
                    let spacing = self.config.block_spacing.spacing(spacing_element);
                    self.ensure_contextual_blank_lines(spacing.top);
                    if matches!(self.config.link_style, LinkStyle::InlineTable) {
                        self.paragraph_link_counter = 0;
                        self.paragraph_links.clear();
                    }
                }

                let is_ordered = start_number.is_some();
                let counter = start_number.unwrap_or(1) as usize;
                self.list_stack.push(ListState {
                    is_ordered,
                    counter,
                    block_start,
                    has_visible_items: false,
                    current_item_start: None,
                    current_item_marker_start: None,
                    current_item_marker_end: None,
                    spacing_element,
                });
                if !self.output.is_empty() && !self.output.ends_with('\n') {
                    self.output.push('\n');
                }
            }
            Tag::Item => {
                if self.list_stack.is_empty() {
                    return Ok(());
                }

                self.reset_explicit_blank_line_streak();

                let indent_level = self.list_stack.len().saturating_sub(1);
                let styled_marker = if let Some(list_state) = self.list_stack.last() {
                    let marker = if list_state.is_ordered {
                        format!("{}. ", list_state.counter)
                    } else {
                        "- ".to_string()
                    };
                    let pretty_level =
                        (!list_state.is_ordered).then_some(indent_level.saturating_add(1));
                    self.styled_list_marker(&marker, pretty_level)
                } else {
                    String::new()
                };

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
                let marker_start = self.output.len();
                self.output.push_str(&styled_marker);

                let marker_end = self.output.len();
                self.commit_pending_heading_placeholder_if_content();

                if let Some(list_state) = self.list_stack.last_mut() {
                    list_state.current_item_start = Some(start_index);
                    list_state.current_item_marker_start = Some(marker_start);
                    list_state.current_item_marker_end = Some(marker_end);

                    if list_state.is_ordered {
                        list_state.counter += 1;
                    }
                }
                self.pending_task_marker = true;
                self.pending_task_marker_buffer.clear();
            }
            Tag::DefinitionList => self.handle_definition_list_start(),
            Tag::DefinitionListTitle => self.handle_definition_title_start(),
            Tag::DefinitionListDefinition => self.handle_definition_description_start(),
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
                    clickable_link_replacements: Vec::new(),
                    inline_url_segments: Vec::new(),
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
                self.close_inline_backticks();
                self.formatting_stack.push(ThemeElement::Emphasis);
            }
            Tag::Strong => {
                self.close_inline_backticks();
                self.formatting_stack.push(ThemeElement::Strong);
            }
            Tag::Strikethrough => {
                self.close_inline_backticks();
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
}
