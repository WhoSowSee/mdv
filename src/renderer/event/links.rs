use super::{
    CapturedReferenceBlock, CowStr, EventRenderer, LinkStyle, LinkTruncationStyle, Result,
    ThemeElement, create_style, wrap_text_with_mode,
};

impl<'a> EventRenderer<'a> {
    pub(super) fn handle_link_start(&mut self, dest_url: CowStr) -> Result<()> {
        // If we are at visual line start (after a soft break or paragraph start),
        // ensure proper indentation/prefix before rendering the link.
        if self.table_state.is_none() {
            let line_start_idx = self.output.rfind('\n').map(|i| i + 1).unwrap_or(0);
            let current_line = &self.output[line_start_idx..];
            if current_line.trim().is_empty() {
                // Normalize any existing whitespace and re-apply consistent prefix/indent
                self.output.truncate(line_start_idx);
                self.push_indent_for_line_start();
            }
        }

        match self.config.link_style {
            LinkStyle::Clickable => {
                // Store URL for clickable link and start collecting link text
                self.link_counter += 1;
                self.link_references.insert(
                    format!("current_{}", self.link_counter),
                    dest_url.to_string(),
                );
                self.current_link_text.clear();
                self.in_link = true;
            }
            LinkStyle::ClickableForced => {
                // Store URL for clickable link with forced underline and start collecting link text
                self.link_counter += 1;
                self.link_references.insert(
                    format!("current_{}", self.link_counter),
                    dest_url.to_string(),
                );
                self.current_link_text.clear();
                self.in_link = true;
            }
            LinkStyle::Hide => {
                // Hide URLs but show link text as normal text
                // Don't set in_link = true, so text is processed normally
            }
            LinkStyle::Inline => {
                // Store URL to add inline after link text and start collecting link text
                self.link_counter += 1;
                self.link_references.insert(
                    format!("current_{}", self.link_counter),
                    dest_url.to_string(),
                );
                self.current_link_text.clear();
                self.in_link = true;
            }
            LinkStyle::InlineTable => {
                // Store URL for paragraph-scoped references and start collecting link text
                self.paragraph_link_counter += 1;
                self.paragraph_links.push((
                    format!("[{}]", self.paragraph_link_counter),
                    dest_url.to_string(),
                ));
                self.current_link_text.clear();
                self.in_link = true;
            }
        }
        Ok(())
    }

    pub(super) fn handle_link_end(&mut self) -> Result<()> {
        match self.config.link_style {
            LinkStyle::Clickable => {
                // For clickable links in tables, just show underlined text instead of OSC 8 sequences
                // to avoid positioning issues with clickable links
                let link_text = &self.current_link_text;
                let formatted_text = self.apply_formatting(link_text);

                if let Some(ref mut table) = self.table_state {
                    // In tables, just underline the link text instead of making it clickable
                    let underlined_text = if !self.config.no_colors {
                        format!("\x1b[4m{}\x1b[0m", formatted_text)
                    } else {
                        formatted_text
                    };

                    table.current_cell.push_str(&underlined_text);
                } else {
                    // For non-table content, use clickable links as before
                    if let Some(url) = self
                        .link_references
                        .get(&format!("current_{}", self.link_counter))
                    {
                        let link_text = &self.current_link_text;

                        // Apply formatting to the link text
                        let formatted_text = self.apply_formatting(link_text);

                        // Create the complete clickable link
                        let final_link = if !self.config.no_colors {
                            format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, formatted_text)
                        } else {
                            // If colors are disabled, just show the text without hyperlink
                            formatted_text
                        };

                        // Process the complete link as a single unit to avoid unwanted line breaks
                        // Check if we need to wrap text
                        let should_wrap = self.config.is_text_wrapping_enabled();

                        if should_wrap {
                            // Check if current line is getting too long (without ANSI codes)
                            let current_line_clean =
                                if let Some(last_newline) = self.output.rfind('\n') {
                                    crate::utils::strip_ansi(&self.output[last_newline + 1..])
                                } else {
                                    crate::utils::strip_ansi(&self.output)
                                };

                            let terminal_width = self.config.get_terminal_width();
                            let current_line_width =
                                crate::utils::display_width(&current_line_clean);
                            let link_width = crate::utils::display_width(link_text);
                            let would_exceed = current_line_width + link_width > terminal_width;

                            // If the complete link would exceed the line width, add a line break before it
                            if would_exceed
                                && current_line_width > 0
                                && !current_line_clean.trim().is_empty()
                            {
                                self.output.push('\n');
                            }
                        }

                        self.output.push_str(&final_link);
                    }
                }
                self.in_link = false;
                self.current_link_text.clear();
            }
            LinkStyle::ClickableForced => {
                // For clickable forced links in tables, just show underlined text instead of OSC 8 sequences
                // to avoid positioning issues with clickable links
                let link_text = &self.current_link_text;
                let formatted_text = self.apply_formatting(link_text);

                if let Some(ref mut table) = self.table_state {
                    // In tables, just underline the link text instead of making it clickable
                    let underlined_text = if !self.config.no_colors {
                        format!("\x1b[4m{}\x1b[0m", formatted_text)
                    } else {
                        formatted_text
                    };

                    table.current_cell.push_str(&underlined_text);
                } else {
                    // For non-table content, use clickable forced links as before
                    if let Some(url) = self
                        .link_references
                        .get(&format!("current_{}", self.link_counter))
                    {
                        let final_link = if !self.config.no_colors {
                            // Wrap the entire OSC 8 construct in underline codes
                            format!(
                                "\x1b[4m\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\\x1b[0m",
                                url,
                                formatted_text // formatted_text already contains bold/italic
                            )
                        } else {
                            // If colors (and styles) are disabled, just output the text
                            formatted_text
                        };

                        // Process the complete link as a single unit to avoid unwanted line breaks
                        // Check if we need to wrap text
                        let should_wrap = self.config.is_text_wrapping_enabled();

                        if should_wrap {
                            // Check if current line is getting too long (without ANSI codes)
                            let current_line_clean =
                                if let Some(last_newline) = self.output.rfind('\n') {
                                    crate::utils::strip_ansi(&self.output[last_newline + 1..])
                                } else {
                                    crate::utils::strip_ansi(&self.output)
                                };

                            let terminal_width = self.config.get_terminal_width();
                            let current_line_width =
                                crate::utils::display_width(&current_line_clean);
                            let link_width = crate::utils::display_width(link_text);
                            let would_exceed = current_line_width + link_width > terminal_width;

                            // If the complete link would exceed the line width, add a line break before it
                            if would_exceed
                                && current_line_width > 0
                                && !current_line_clean.trim().is_empty()
                            {
                                self.output.push('\n');
                            }
                        }

                        self.output.push_str(&final_link);
                    }
                }
                self.in_link = false;
                self.current_link_text.clear();
            }
            LinkStyle::Hide => {
                // Nothing to do - link text was processed as normal text
            }
            LinkStyle::Inline => {
                // For Inline mode, process link text with normal wrapping, then add URL
                let current_link_text = self.current_link_text.clone();
                let url = self
                    .link_references
                    .get(&format!("current_{}", self.link_counter))
                    .cloned();

                if let Some(url) = url {
                    // Check if we're in a table cell
                    if let Some(ref mut table) = self.table_state {
                        // For tables, format as single unit
                        let formatted_link_text = if !self.config.no_colors {
                            format!("\x1b[4m{}\x1b[0m", current_link_text)
                        } else {
                            current_link_text.clone()
                        };
                        let url_part = format!("({})", url);
                        let style = create_style(self.theme, ThemeElement::Link);
                        let styled_url = style.apply(&url_part, self.config.no_colors);

                        table.inline_references.push((url_part.clone(), styled_url));
                        table.current_cell.push_str(&formatted_link_text);
                        table.current_cell.push_str(&url_part);
                    } else {
                        // Process link text with underline formatting and normal wrapping logic
                        self.process_underlined_text_with_wrapping(&current_link_text)?;
                        // Safety: if last visual line overflowed by a single dangling character, fix it
                        self.enforce_width_on_current_line();

                        // Now add the URL part
                        let url_part = format!("({})", url);

                        // Check if URL needs wrapping or truncation
                        let should_wrap = self.config.is_text_wrapping_enabled();

                        if should_wrap {
                            let current_line_clean =
                                if let Some(last_newline) = self.output.rfind('\n') {
                                    crate::utils::strip_ansi(&self.output[last_newline + 1..])
                                } else {
                                    crate::utils::strip_ansi(&self.output)
                                };

                            let terminal_width = self.config.get_terminal_width();
                            let current_line_width =
                                crate::utils::display_width(&current_line_clean);
                            let url_part_width = crate::utils::display_width(&url_part);

                            // Check truncation style for Inline mode
                            match self.config.link_truncation {
                                LinkTruncationStyle::Cut => {
                                    // Precisely fit the URL display into the remaining space on the current line.
                                    let available_width =
                                        terminal_width.saturating_sub(current_line_width);

                                    if available_width >= url_part_width {
                                        // URL fits entirely on the current line
                                        let style = create_style(self.theme, ThemeElement::Link);
                                        let styled_url =
                                            style.apply(&url_part, self.config.no_colors);
                                        let clickable_url =
                                            self.make_clickable_link(&styled_url, &url);
                                        self.output.push_str(&clickable_url);
                                        self.enforce_width_on_current_line();
                                    } else if available_width > 2 {
                                        // Space available only for a truncated form inside parentheses
                                        let available_for_url = available_width.saturating_sub(2); // -2 for parentheses
                                        let truncated_display = self
                                            .truncate_url_with_ellipsis(&url, available_for_url);
                                        let truncated_url_part = format!("({})", truncated_display);
                                        let style = create_style(self.theme, ThemeElement::Link);
                                        let styled_truncated =
                                            style.apply(&truncated_url_part, self.config.no_colors);
                                        let clickable_truncated =
                                            self.make_clickable_link(&styled_truncated, &url);
                                        self.output.push_str(&clickable_truncated);
                                    } else {
                                        // Not enough space left on this visual line – break and place URL at the start
                                        // of the next line with proper indentation, then fit it there.
                                        self.output.push('\n');

                                        if self.content_indent > 0 {
                                            self.output.push_str(&" ".repeat(self.content_indent));
                                        }
                                        if self.blockquote_level > 0 {
                                            let prefix = self.render_blockquote_prefix();
                                            self.output.push_str(&prefix);
                                        }

                                        // Effective width for the new line considering indentation
                                        let mut effective_width_for_url = terminal_width;
                                        if self.content_indent > 0 {
                                            effective_width_for_url = effective_width_for_url
                                                .saturating_sub(self.content_indent);
                                        }
                                        if self.blockquote_level > 0 {
                                            let prefix_width = self.blockquote_level + 1; // │ + space
                                            effective_width_for_url = effective_width_for_url
                                                .saturating_sub(prefix_width);
                                        }

                                        let available_for_url =
                                            effective_width_for_url.saturating_sub(2);
                                        let truncated_display = self
                                            .truncate_url_with_ellipsis(&url, available_for_url);
                                        let truncated_url_part = format!("({})", truncated_display);
                                        let style = create_style(self.theme, ThemeElement::Link);
                                        let styled_truncated =
                                            style.apply(&truncated_url_part, self.config.no_colors);
                                        let clickable_truncated =
                                            self.make_clickable_link(&styled_truncated, &url);
                                        self.output.push_str(&clickable_truncated);
                                    }
                                }
                                LinkTruncationStyle::None => {
                                    // No truncation - make URL clickable even if it overflows
                                    let style = create_style(self.theme, ThemeElement::Link);
                                    let styled_url = style.apply(&url_part, self.config.no_colors);
                                    let clickable_url = self.make_clickable_link(&styled_url, &url);
                                    self.output.push_str(&clickable_url);
                                }
                                LinkTruncationStyle::Wrap => {
                                    // Flexible wrapping: place as much as fits on the current line,
                                    // then continue on the next line with proper indentation.
                                    if current_line_width + url_part_width <= terminal_width {
                                        // Fits entirely on the current line
                                        let style = create_style(self.theme, ThemeElement::Link);
                                        let styled_url =
                                            style.apply(&url_part, self.config.no_colors);
                                        let clickable_url =
                                            self.make_clickable_link(&styled_url, &url);
                                        self.output.push_str(&clickable_url);
                                    } else {
                                        // Split URL text into two parts: the remainder that fits on this line,
                                        // and the rest that goes to the next line(s).
                                        let mut taken = String::new();
                                        let mut remaining = String::new();
                                        let mut acc = 0usize;
                                        for ch in url_part.chars() {
                                            let w = crate::utils::display_width(&ch.to_string());
                                            if acc + w
                                                <= terminal_width.saturating_sub(current_line_width)
                                            {
                                                taken.push(ch);
                                                acc += w;
                                            } else {
                                                remaining.push(ch);
                                            }
                                        }

                                        // Add the part that fits to the current line
                                        if !taken.is_empty() {
                                            let style =
                                                create_style(self.theme, ThemeElement::Link);
                                            let styled_taken =
                                                style.apply(&taken, self.config.no_colors);
                                            let clickable_taken =
                                                self.make_clickable_link(&styled_taken, &url);
                                            self.output.push_str(&clickable_taken);
                                        }

                                        // If anything remains, break the line and render the rest with indentation
                                        if !remaining.is_empty() {
                                            // New visual line for the rest of the URL
                                            self.push_newline_with_context();

                                            // Wrap the remaining part for subsequent lines
                                            let style =
                                                create_style(self.theme, ThemeElement::Link);
                                            let styled_remaining =
                                                style.apply(&remaining, self.config.no_colors);
                                            let wrapped_url =
                                                self.wrap_url_with_indentation(&styled_remaining);
                                            let clickable_wrapped =
                                                self.make_clickable_wrapped_url(&url, &wrapped_url);
                                            self.output.push_str(&clickable_wrapped);
                                            self.enforce_width_on_current_line();
                                        }
                                    }
                                }
                            }
                        } else {
                            // No wrapping, but still ensure we do not exceed terminal width
                            match self.config.link_truncation {
                                LinkTruncationStyle::Cut => {
                                    let terminal_width = self.config.get_terminal_width();
                                    let current_line_clean = if let Some(last_newline) =
                                        self.output.rfind('\n')
                                    {
                                        crate::utils::strip_ansi(&self.output[last_newline + 1..])
                                    } else {
                                        crate::utils::strip_ansi(&self.output)
                                    };
                                    let current_line_width =
                                        crate::utils::display_width(&current_line_clean);
                                    let available_width =
                                        terminal_width.saturating_sub(current_line_width);
                                    let url_part_width = crate::utils::display_width(&url_part);

                                    if available_width >= url_part_width {
                                        let style = create_style(self.theme, ThemeElement::Link);
                                        let styled_url =
                                            style.apply(&url_part, self.config.no_colors);
                                        let clickable_url =
                                            self.make_clickable_link(&styled_url, &url);
                                        self.output.push_str(&clickable_url);
                                        self.enforce_width_on_current_line();
                                    } else if available_width > 2 {
                                        let available_for_url = available_width.saturating_sub(2);
                                        let truncated_display = self
                                            .truncate_url_with_ellipsis(&url, available_for_url);
                                        let truncated_url_part = format!("({})", truncated_display);
                                        let style = create_style(self.theme, ThemeElement::Link);
                                        let styled_truncated =
                                            style.apply(&truncated_url_part, self.config.no_colors);
                                        let clickable_truncated =
                                            self.make_clickable_link(&styled_truncated, &url);
                                        self.output.push_str(&clickable_truncated);
                                        self.enforce_width_on_current_line();
                                    } else {
                                        // Not enough space even for parentheses; show minimal clickable marker if possible
                                        if available_width > 0 {
                                            let style =
                                                create_style(self.theme, ThemeElement::Link);
                                            let marker = style.apply("…", self.config.no_colors);
                                            let clickable_marker =
                                                self.make_clickable_link(&marker, &url);
                                            self.output.push_str(&clickable_marker);
                                        }
                                    }
                                }
                                _ => {
                                    // Just add clickable URL without wrapping or truncation
                                    let style = create_style(self.theme, ThemeElement::Link);
                                    let styled_url = style.apply(&url_part, self.config.no_colors);
                                    let clickable_url = self.make_clickable_link(&styled_url, &url);
                                    self.output.push_str(&clickable_url);
                                    self.enforce_width_on_current_line();
                                }
                            }
                        }
                    }
                }
                self.in_link = false;
                self.current_link_text.clear();
            }

            LinkStyle::InlineTable => {
                // InlineTable needs special handling in tables to avoid duplicating the
                // link text outside the table. If we're inside a table cell, write the
                // entire link (underlined text + reference) directly into the cell and
                // skip any rendering to the main output buffer.

                if let Some(ref mut table) = self.table_state {
                    let reference_text = format!("[{}]", self.paragraph_link_counter);
                    let style = create_style(self.theme, ThemeElement::Link);
                    let styled_reference = style.apply(&reference_text, self.config.no_colors);

                    let formatted_link_text = if !self.config.no_colors {
                        format!("\x1b[4m{}\x1b[0m", self.current_link_text)
                    } else {
                        self.current_link_text.clone()
                    };

                    table
                        .inline_references
                        .push((reference_text.clone(), styled_reference));
                    table.current_cell.push_str(&formatted_link_text);
                    table.current_cell.push_str(&reference_text);
                } else {
                    // 1) Render the link text underlined with proper wrapping
                    let link_text = self.current_link_text.clone();
                    self.process_underlined_text_with_wrapping(&link_text)?;

                    // 2) Append the reference number after the text (wrap if needed)
                    let reference_text = format!("[{}]", self.paragraph_link_counter);
                    let style = create_style(self.theme, ThemeElement::Link);
                    let styled_reference = style.apply(&reference_text, self.config.no_colors);

                    // Decide if reference fits on current line
                    let current_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
                        crate::utils::strip_ansi(&self.output[last_newline + 1..])
                    } else {
                        crate::utils::strip_ansi(&self.output)
                    };
                    let terminal_width = self.config.get_terminal_width();
                    let current_line_width = crate::utils::display_width(&current_line_clean);
                    let reference_width = crate::utils::display_width(&reference_text);

                    if self.config.is_text_wrapping_enabled()
                        && current_line_width + reference_width > terminal_width
                    {
                        self.push_newline_with_context();
                    }
                    self.output.push_str(&styled_reference);
                }

                self.in_link = false;
                self.current_link_text.clear();
            }
        }
        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }

    pub(super) fn add_paragraph_link_references(&mut self) {
        let in_list = !self.list_stack.is_empty();
        let in_table = false; // Regular call, not from table context
        self.add_paragraph_link_references_with_trailing_newline(true, in_list, in_table);
    }

    pub(super) fn add_paragraph_link_references_for_table(&mut self) {
        let in_list = !self.list_stack.is_empty();
        let in_table = true; // Called from table context
        self.add_paragraph_link_references_with_trailing_newline(true, in_list, in_table);
    }

    pub(super) fn add_paragraph_link_references_with_trailing_newline(
        &mut self,
        add_trailing_newline: bool,
        in_list: bool,
        in_table: bool,
    ) {
        if self.paragraph_links.is_empty() {
            return;
        }

        let style = create_style(self.theme, ThemeElement::Link);

        let mut styled_blocks: Vec<Vec<String>> = Vec::new();

        for (reference, url) in &self.paragraph_links {
            let link_line = format!("{} {}", reference, url);

            let wrapped_link = if self.config.is_text_wrapping_enabled() {
                self.wrap_link_line(&link_line)
            } else {
                link_line
            };

            let styled_lines: Vec<String> = wrapped_link
                .lines()
                .map(|line| {
                    let clickable_line = self.make_clickable_link(line, url);
                    style.apply(&clickable_line, self.config.no_colors)
                })
                .collect();

            styled_blocks.push(styled_lines);
        }

        if self.plaintext_code_block_depth > 0 {
            let captured_lines: Vec<String> = styled_blocks
                .iter()
                .flat_map(|lines| lines.clone())
                .collect();

            self.captured_reference_blocks.push(CapturedReferenceBlock {
                lines: captured_lines,
                add_trailing_newline,
                in_list,
            });

            self.paragraph_links.clear();
            return;
        }

        // Add empty line before link references for consistent formatting
        // Check if we already have a newline at the end, if so add only one more
        if self.output.ends_with('\n') {
            self.output.push('\n');
        } else {
            self.output.push('\n');
            self.output.push('\n');
        }
        for (i, styled_lines) in styled_blocks.iter().enumerate() {
            for (line_idx, styled_line) in styled_lines.iter().enumerate() {
                if self.content_indent > 0 && !in_table {
                    self.output.push_str(&" ".repeat(self.content_indent));
                }

                self.output.push_str(styled_line);

                if line_idx < styled_lines.len() - 1 || i < styled_blocks.len() - 1 {
                    self.output.push('\n');
                }
            }
        }

        // Add trailing newline after the link block if requested
        if add_trailing_newline {
            self.output.push('\n');
            // Add extra newline only when in list for proper spacing
            if in_list {
                self.output.push('\n');
            }
        }

        // Clear the paragraph links after adding them
        self.paragraph_links.clear();
    }

    /// Wrap a link line (reference + URL) with proper handling of URL breaking
    pub(super) fn wrap_link_line(&self, link_line: &str) -> String {
        let terminal_width = self.config.get_terminal_width();

        // Link reference lines are printed later with a leading content indentation
        // (self.content_indent spaces). That indentation must be accounted for when
        // deciding how much of the URL can fit on a visual line, otherwise we risk
        // overflowing by 1–N cells and the trailing "..." gets visually clipped to
        // ".." or ".". Compute an effective width for the visible content area.
        let effective_width = terminal_width.saturating_sub(self.content_indent);

        // Don't wrap if width is too small
        if effective_width < 20 {
            return link_line.to_string();
        }

        // Check if the line fits without wrapping
        if crate::utils::display_width(link_line) <= effective_width {
            return link_line.to_string();
        }

        // Split the link line into reference and URL parts
        if let Some(space_pos) = link_line.find(' ') {
            let reference = &link_line[..space_pos];
            let url = &link_line[space_pos + 1..];

            // Calculate available width for URL (accounting for reference + space)
            let reference_width = crate::utils::display_width(reference) + 1; // +1 for space
            let available_width = effective_width.saturating_sub(reference_width);

            // If URL fits in available width, no wrapping needed
            if crate::utils::display_width(url) <= available_width {
                return link_line.to_string();
            }

            // Check truncation style - only apply for InlineTable mode
            if matches!(self.config.link_style, LinkStyle::InlineTable) {
                match self.config.link_truncation {
                    LinkTruncationStyle::Cut => {
                        // Cut the URL and add "..." if it doesn't fit
                        let truncated_url = self.truncate_url_with_ellipsis(url, available_width);
                        return format!("{} {}", reference, truncated_url);
                    }
                    LinkTruncationStyle::None => {
                        // No truncation - return the link as is, even if it overflows
                        return link_line.to_string();
                    }
                    LinkTruncationStyle::Wrap => {
                        // Use the original wrapping logic
                    }
                }
            }

            // Wrap the URL part with proper indentation based on reference length
            let wrapped_url = self.wrap_url_with_reference(
                url,
                available_width,
                effective_width,
                reference_width,
            );

            // Combine reference with wrapped URL
            format!("{} {}", reference, wrapped_url)
        } else {
            // Fallback: wrap the entire line as text
            let wrap_mode = self.config.text_wrap_mode();
            crate::utils::wrap_text_with_mode(link_line, terminal_width, wrap_mode)
        }
    }

    /// Wrap a URL with smart breaking at appropriate characters
    pub(super) fn wrap_url_with_reference(
        &self,
        url: &str,
        first_line_width: usize,
        continuation_width: usize,
        reference_width: usize,
    ) -> String {
        if crate::utils::display_width(url) <= first_line_width {
            return url.to_string();
        }

        let mut result = String::new();
        let mut current_line = String::new();
        let mut current_width = 0;
        let mut is_first_line = true;

        // Characters that are good breaking points in URLs
        let good_break_chars = ['/', '?', '&', '=', '-', '_', '.', ':', '#'];

        // Calculate the indent for continuation lines based on the actual reference width
        // This creates the exact same indentation as the reference part
        let continuation_indent = " ".repeat(reference_width);

        let chars: Vec<char> = url.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];
            let char_width = crate::utils::display_width(&ch.to_string());
            let max_width = if is_first_line {
                first_line_width
            } else {
                continuation_width.saturating_sub(reference_width)
            };

            // Check if adding this character would exceed the line width
            if current_width + char_width > max_width && !current_line.is_empty() {
                // Look for a good breaking point in the current line
                if let Some(break_pos) = self.find_url_break_point(&current_line, &good_break_chars)
                {
                    // Break at the good point
                    let (line_part, remaining) = current_line.split_at(break_pos);
                    result.push_str(line_part);
                    result.push('\n');

                    // Add indent for continuation line and start with remaining characters plus current character
                    result.push_str(&continuation_indent);
                    current_line = format!("{}{}", remaining, ch);
                    current_width = crate::utils::display_width(&current_line);
                } else {
                    // No good breaking point found, force break
                    result.push_str(&current_line);
                    result.push('\n');

                    // Add indent for continuation line and start with current character
                    result.push_str(&continuation_indent);
                    current_line = ch.to_string();
                    current_width = crate::utils::display_width(&current_line);
                }
                is_first_line = false;
            } else {
                // Add character to current line
                current_line.push(ch);
                current_width += char_width;
            }

            i += 1;
        }

        // Add remaining characters
        if !current_line.is_empty() {
            result.push_str(&current_line);
        }

        result
    }

    /// Find the best breaking point in a URL segment
    pub(super) fn wrap_text_for_output(&self, text: &str) -> String {
        // Check if wrapping is disabled
        if !self.config.is_text_wrapping_enabled() {
            return text.to_string();
        }

        let terminal_width = self.config.get_terminal_width();

        // Don't wrap if width is too small or text is very short
        if terminal_width < 20 || text.trim().len() < 10 {
            return text.to_string();
        }

        // Calculate effective width considering heading indentation and blockquote prefix
        let mut effective_width = terminal_width;

        // Account for heading indentation
        if self.content_indent > 0 {
            effective_width = effective_width.saturating_sub(self.content_indent);
        }

        // Account for blockquote prefix if in blockquote
        if self.current_indent > 0 {
            // Each level adds one │ character plus one space
            let prefix_width = self.blockquote_level + 1; // │ symbols + space
            effective_width = effective_width.saturating_sub(prefix_width);
        }

        // Only wrap if we have a reasonable width (minimum 10 characters for deep nesting)
        if effective_width < 10 {
            return text.to_string();
        }

        // Determine wrapping mode
        let wrap_mode = self.config.text_wrap_mode();

        // Use our text wrapping utility
        // For blockquotes, don't add indentation here - we'll add the │ prefix manually
        wrap_text_with_mode(text, effective_width, wrap_mode)
    }

    /// Wrap URL text with proper indentation for each line
    pub(super) fn wrap_url_with_indentation(&self, text: &str) -> String {
        let wrapped = self.wrap_text_for_output(text);

        // If the text wasn't actually wrapped (no newlines), return as is
        if !wrapped.contains('\n') {
            return wrapped;
        }

        // Split into lines and add indentation to continuation lines
        let lines: Vec<&str> = wrapped.split('\n').collect();
        let mut result = String::new();

        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                result.push('\n');

                // Add proper indentation for continuation lines
                if self.blockquote_level > 0 {
                    // Heading/content indent
                    if self.content_indent > 0 {
                        result.push_str(&" ".repeat(self.content_indent));
                    }
                    // Blockquote prefix
                    for _ in 0..self.blockquote_level {
                        result.push('│');
                    }
                    result.push(' ');
                    // If inside a list, align to list content (exclude heading indent already added)
                    if !self.list_stack.is_empty() {
                        let full_list_indent = self.calculate_list_content_indent();
                        let additional = full_list_indent.saturating_sub(self.content_indent);
                        if additional > 0 {
                            result.push_str(&" ".repeat(additional));
                        }
                    }
                } else if !self.list_stack.is_empty() {
                    let list_content_indent = self.calculate_list_content_indent();
                    result.push_str(&" ".repeat(list_content_indent));
                } else if self.content_indent > 0 {
                    result.push_str(&" ".repeat(self.content_indent));
                }
            }
            result.push_str(line);
        }

        result
    }

    pub(super) fn find_url_break_point(
        &self,
        line: &str,
        good_break_chars: &[char],
    ) -> Option<usize> {
        // Look for good breaking points from right to left (prefer breaking later)
        for (i, ch) in line.char_indices().rev() {
            if good_break_chars.contains(&ch) {
                // Break after the special character (not before)
                return Some(i + ch.len_utf8());
            }
        }
        None
    }

    /// Truncate URL with ellipsis if it doesn't fit in available width
    pub(super) fn truncate_url_with_ellipsis(&self, url: &str, available_width: usize) -> String {
        // Always ensure the returned string's display width is <= available_width.
        // Use three-dot ellipsis when possible, otherwise fit the number of dots
        // that can be displayed (including zero when there is no space at all).
        if available_width == 0 {
            return String::new();
        }

        // When very little space remains, prefer a minimal visual indicator that fits.
        if available_width <= 2 {
            return ".".repeat(available_width);
        }

        let ellipsis = "...";
        let ellipsis_width = 3; // display width of "..."

        // If URL already fits, return as is
        if crate::utils::display_width(url) <= available_width {
            return url.to_string();
        }

        // Calculate maximum width for URL content (leaving space for ellipsis)
        let max_url_width = available_width.saturating_sub(ellipsis_width);

        // Find the best truncation point
        let mut truncated = String::new();
        let mut current_width = 0;

        for ch in url.chars() {
            let char_width = crate::utils::display_width(&ch.to_string());
            if current_width + char_width > max_url_width {
                break;
            }
            truncated.push(ch);
            current_width += char_width;
        }

        // Add ellipsis
        format!("{}{}", truncated, ellipsis)
    }

    /// Make a text line clickable by wrapping it in terminal hyperlink escape sequences
    pub(super) fn make_clickable_link(&self, text: &str, url: &str) -> String {
        if self.config.no_colors {
            // If colors are disabled, don't add hyperlink sequences
            return text.to_string();
        }

        // Use OSC 8 hyperlink escape sequence to make text clickable
        // Format: \e]8;;URL\e\\TEXT\e]8;;\e\\
        format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, text)
    }

    /// Create a clickable wrapped URL where each part opens the full original URL
    pub(super) fn make_clickable_wrapped_url(
        &self,
        original_url: &str,
        styled_wrapped_url: &str,
    ) -> String {
        if self.config.no_colors {
            return styled_wrapped_url.to_string();
        }

        // Split the wrapped URL by newlines and make each part clickable
        let lines: Vec<&str> = styled_wrapped_url.split('\n').collect();
        let mut result = String::new();

        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                result.push('\n');
            }

            // Get clean text to check if line is empty
            let clean_line = crate::utils::strip_ansi(line);
            if !clean_line.trim().is_empty() {
                // Apply link styling to clean text first
                let style = create_style(self.theme, crate::theme::ThemeElement::Link);
                let styled_clean_line = style.apply(&clean_line, self.config.no_colors);
                // Then make the styled text clickable
                let clickable_line = self.make_clickable_link(&styled_clean_line, original_url);
                result.push_str(&clickable_line);
            } else {
                result.push_str(line);
            }
        }

        result
    }
    /// Ensure the last visual line does not exceed the terminal width.
    /// If it does, break the line at the last space and add proper indentation/prefixes.
    pub(super) fn enforce_width_on_current_line(&mut self) {
        let terminal_width = self.config.get_terminal_width();
        let start = self.output.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let current_line_raw = &self.output[start..];
        let clean = crate::utils::strip_ansi(current_line_raw);
        let width = crate::utils::display_width(&clean);

        if width <= terminal_width {
            return;
        }

        // Find last space to break at; avoid breaking at the very first
        // leading space (indentation) which would produce a blank line.
        if let Some(space_rel_idx) = current_line_raw.rfind(' ') {
            if space_rel_idx == 0 {
                return;
            }
            // Build indentation for continuation line
            let mut indent = String::new();
            if self.blockquote_level > 0 {
                if self.content_indent > 0 {
                    indent.push_str(&" ".repeat(self.content_indent));
                }
                let prefix = self.render_blockquote_prefix();
                indent.push_str(&prefix);
                if !self.list_stack.is_empty() {
                    let full_list_indent = self.calculate_list_content_indent();
                    let additional = full_list_indent.saturating_sub(self.content_indent);
                    if additional > 0 {
                        indent.push_str(&" ".repeat(additional));
                    }
                }
            } else if !self.list_stack.is_empty() {
                let list_content_indent = self.calculate_list_content_indent();
                indent.push_str(&" ".repeat(list_content_indent));
            } else if self.content_indent > 0 {
                indent.push_str(&" ".repeat(self.content_indent));
            }

            // Replace the space with a newline + indent
            let insert = format!("\n{}", indent);
            let abs_idx = start + space_rel_idx;
            self.output.replace_range(abs_idx..abs_idx + 1, &insert);
        }
    }
}
