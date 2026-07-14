use super::core::CalloutState;
use super::{
    CapturedReferenceBlock, CowStr, DeferredLinkReferenceBlock, EventRenderer, LinkStyle,
    LinkTruncationStyle, Result, TableInlineUrlSegment, TableInlineUrlTarget, TableState,
    ThemeElement, create_style, wrap_text_with_mode,
};

const TABLE_REFERENCE_WRAP_DELIMITER: char = '\u{200B}';

fn build_underlined_table_link_replacement(link_text: &str, no_colors: bool) -> Option<String> {
    if no_colors || link_text.is_empty() {
        None
    } else {
        Some(format!("\x1b[4m{}\x1b[24m", link_text))
    }
}

fn build_clickable_underlined_table_link_replacement(
    link_text: &str,
    url: &str,
    no_colors: bool,
) -> Option<String> {
    if no_colors || link_text.is_empty() {
        None
    } else {
        Some(format!(
            "\x1b]8;;{}\x1b\\\x1b[4m{}\x1b[24m\x1b]8;;\x1b\\",
            url, link_text
        ))
    }
}

fn push_table_link_with_replacement(
    table: &mut TableState,
    link_text: &str,
    replacement: Option<String>,
) {
    if link_text.is_empty() {
        return;
    }

    if let Some(styled) = replacement {
        table
            .inline_references
            .push((link_text.to_string(), styled));
    }

    table.current_cell.push_str(link_text);
}

fn push_underlined_table_link(table: &mut TableState, link_text: &str, no_colors: bool) {
    let replacement = build_underlined_table_link_replacement(link_text, no_colors);
    push_table_link_with_replacement(table, link_text, replacement);
}

fn push_wrappable_table_reference(cell: &mut String, reference_text: &str) {
    if reference_text.is_empty() {
        return;
    }

    // Insert a zero-width delimiter so comfy-table can wrap before `[N]` when needed
    // without changing visible content width.
    let needs_separator = cell
        .chars()
        .last()
        .map(|ch| !ch.is_whitespace() && ch != TABLE_REFERENCE_WRAP_DELIMITER)
        .unwrap_or(false);

    if needs_separator {
        cell.push(TABLE_REFERENCE_WRAP_DELIMITER);
    }

    cell.push_str(reference_text);
}

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
                let in_table = self.table_state.is_some();
                if let Some(CalloutState::Active(info)) = self.callout_stack.last_mut() {
                    if in_table {
                        self.paragraph_link_counter += 1;
                        self.paragraph_links.push((
                            format!("[{}]", self.paragraph_link_counter),
                            dest_url.to_string(),
                        ));
                    } else {
                        info.inline_link_counter += 1;
                        let reference = format!("[{}]", info.inline_link_counter);
                        info.inline_links.push((reference, dest_url.to_string()));
                    }
                } else {
                    // Store URL for paragraph-scoped references and start collecting link text
                    self.paragraph_link_counter += 1;
                    self.paragraph_links.push((
                        format!("[{}]", self.paragraph_link_counter),
                        dest_url.to_string(),
                    ));
                }
                self.current_link_text.clear();
                self.in_link = true;
            }
            LinkStyle::EndTable => {
                // Store URL for document-scoped references and start collecting link text
                self.paragraph_link_counter += 1;
                self.document_links.push((
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
        if self.table_state.is_none() && !matches!(self.config.link_style, LinkStyle::Hide) {
            self.note_paragraph_content();
        }

        match self.config.link_style {
            LinkStyle::Clickable => {
                let link_text = self.current_link_text.clone();
                let current_link_key = format!("current_{}", self.link_counter);
                let link_url = self.link_references.get(&current_link_key).cloned();

                if let Some(ref mut table) = self.table_state {
                    // Keep table width calculation ANSI-free, then inject clickable styling
                    // into the rendered fragments after table layout.
                    let replacement = link_url
                        .as_deref()
                        .and_then(|url| {
                            build_clickable_underlined_table_link_replacement(
                                &link_text,
                                url,
                                self.config.no_colors,
                            )
                        })
                        .or_else(|| {
                            build_underlined_table_link_replacement(
                                &link_text,
                                self.config.no_colors,
                            )
                        });
                    push_table_link_with_replacement(table, &link_text, replacement);
                } else {
                    // For non-table content, use clickable links as before
                    if let Some(url) = link_url.as_deref() {
                        // Apply formatting to the link text
                        let formatted_text = self.apply_formatting(&link_text);

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

                            let terminal_width = self.effective_text_width();
                            let current_line_width =
                                crate::utils::display_width(&current_line_clean);
                            let link_width = crate::utils::display_width(&link_text);
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
                let link_text = self.current_link_text.clone();
                let formatted_text = self.apply_formatting(&link_text);
                let current_link_key = format!("current_{}", self.link_counter);
                let link_url = self.link_references.get(&current_link_key).cloned();

                if let Some(ref mut table) = self.table_state {
                    // Keep table width calculation ANSI-free, then inject clickable styling
                    // into the rendered fragments after table layout.
                    let replacement = link_url
                        .as_deref()
                        .and_then(|url| {
                            build_clickable_underlined_table_link_replacement(
                                &link_text,
                                url,
                                self.config.no_colors,
                            )
                        })
                        .or_else(|| {
                            build_underlined_table_link_replacement(
                                &link_text,
                                self.config.no_colors,
                            )
                        });
                    push_table_link_with_replacement(table, &link_text, replacement);
                } else {
                    // For non-table content, use clickable forced links as before
                    if let Some(url) = link_url.as_deref() {
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

                            let terminal_width = self.effective_text_width();
                            let current_line_width =
                                crate::utils::display_width(&current_line_clean);
                            let link_width = crate::utils::display_width(&link_text);
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
                        // For tables, apply underline to the exact link fragment after layout.
                        push_underlined_table_link(
                            table,
                            &current_link_text,
                            self.config.no_colors,
                        );
                        let url_part = format!("({})", url);
                        let style = create_style(self.theme, ThemeElement::Link);
                        let styled_url = style.apply(&url_part, self.config.no_colors);

                        if matches!(self.config.link_truncation, LinkTruncationStyle::TableCut) {
                            let target = if table.in_header {
                                TableInlineUrlTarget::Header {
                                    column_index: table.current_row.len(),
                                }
                            } else {
                                TableInlineUrlTarget::Row {
                                    row_index: table.rows.len(),
                                    column_index: table.current_row.len(),
                                }
                            };

                            table.inline_url_segments.push(TableInlineUrlSegment {
                                target,
                                url: url.clone(),
                                url_part: url_part.clone(),
                            });
                        }

                        table.inline_references.push((url_part.clone(), styled_url));
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

                            let terminal_width = self.effective_text_width();
                            let current_line_width =
                                crate::utils::display_width(&current_line_clean);
                            let url_part_width = crate::utils::display_width(&url_part);

                            // Check truncation style for Inline mode
                            match self.config.link_truncation {
                                LinkTruncationStyle::Cut | LinkTruncationStyle::TableCut => {
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
                                        self.push_indent_for_line_start();

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
                                LinkTruncationStyle::Cut | LinkTruncationStyle::TableCut => {
                                    let terminal_width = self.effective_text_width();
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
                let reference_index = if self.table_state.is_some() {
                    self.paragraph_link_counter
                } else {
                    match self.callout_stack.last() {
                        Some(CalloutState::Active(info)) => info.inline_link_counter,
                        _ => self.paragraph_link_counter,
                    }
                };
                let reference_text = format!("[{}]", reference_index);

                if let Some(ref mut table) = self.table_state {
                    let style = create_style(self.theme, ThemeElement::Link);
                    let styled_reference = style.apply(&reference_text, self.config.no_colors);

                    push_underlined_table_link(
                        table,
                        &self.current_link_text,
                        self.config.no_colors,
                    );

                    table
                        .inline_references
                        .push((reference_text.clone(), styled_reference));
                    push_wrappable_table_reference(&mut table.current_cell, &reference_text);
                } else {
                    // 1) Render the link text underlined with proper wrapping
                    let link_text = self.current_link_text.trim().to_string();
                    if !link_text.is_empty() {
                        self.process_underlined_text_with_wrapping(&link_text)?;
                    }

                    // 2) Append the reference number after the text (wrap if needed)
                    let style = create_style(self.theme, ThemeElement::Link);
                    let styled_reference = style.apply(&reference_text, self.config.no_colors);

                    // Decide if reference fits on current line
                    let current_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
                        crate::utils::strip_ansi(&self.output[last_newline + 1..])
                    } else {
                        crate::utils::strip_ansi(&self.output)
                    };
                    let terminal_width = self.effective_text_width();
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
            LinkStyle::EndTable => {
                // Behave like InlineTable for inline markers but collect references for document-level table.
                if let Some(ref mut table) = self.table_state {
                    let reference_text = format!("[{}]", self.paragraph_link_counter);
                    let style = create_style(self.theme, ThemeElement::Link);
                    let styled_reference = style.apply(&reference_text, self.config.no_colors);

                    push_underlined_table_link(
                        table,
                        &self.current_link_text,
                        self.config.no_colors,
                    );

                    table
                        .inline_references
                        .push((reference_text.clone(), styled_reference));
                    push_wrappable_table_reference(&mut table.current_cell, &reference_text);
                } else {
                    let link_text = self.current_link_text.trim().to_string();
                    if !link_text.is_empty() {
                        self.process_underlined_text_with_wrapping(&link_text)?;
                    }

                    let reference_text = format!("[{}]", self.paragraph_link_counter);
                    let style = create_style(self.theme, ThemeElement::Link);
                    let styled_reference = style.apply(&reference_text, self.config.no_colors);

                    let current_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
                        crate::utils::strip_ansi(&self.output[last_newline + 1..])
                    } else {
                        crate::utils::strip_ansi(&self.output)
                    };
                    let terminal_width = self.effective_text_width();
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
        self.add_paragraph_link_references_with_trailing_newline(true, in_list, in_table, 0);
    }

    pub(super) fn add_paragraph_link_references_for_table(&mut self, table_indent: usize) {
        let in_list = !self.list_stack.is_empty();
        let in_table = true; // Called from table context
        self.add_paragraph_link_references_with_trailing_newline(
            true,
            in_list,
            in_table,
            table_indent,
        );
    }

    pub(super) fn render_link_reference_blocks(
        &mut self,
        links: &[(String, String)],
        add_trailing_newline: bool,
        in_list: bool,
        in_table: bool,
        table_indent: usize,
    ) {
        if links.is_empty() {
            return;
        }

        let reference_indent = if in_table {
            table_indent
        } else {
            self.content_indent
        };
        let reference_prefix = if in_table && self.blockquote_level > 0 {
            self.current_line_prefix()
        } else {
            String::new()
        };
        let pretty_callout_padding = if matches!(
            self.config.callout_style.style,
            crate::cli::CalloutStyle::Pretty
        ) && self
            .callout_stack
            .iter()
            .any(|state| matches!(state, CalloutState::Active(_)))
        {
            2
        } else {
            0
        };
        let reference_wrap_padding = reference_indent
            .saturating_add(crate::utils::display_width(&crate::utils::strip_ansi(
                &reference_prefix,
            )))
            .saturating_add(pretty_callout_padding);
        let styled_blocks = self.build_styled_reference_blocks(links, reference_wrap_padding);

        if self.plaintext_code_block_depth > 0 {
            if in_table {
                let captured_lines: Vec<String> = styled_blocks
                    .iter()
                    .flat_map(|lines| lines.clone())
                    .collect();

                self.captured_reference_blocks.push(CapturedReferenceBlock {
                    lines: captured_lines,
                    add_trailing_newline,
                });
            } else {
                self.deferred_reference_blocks
                    .push(DeferredLinkReferenceBlock {
                        links: links.to_vec(),
                        add_trailing_newline,
                    });
            }

            return;
        }

        // Add empty line before link references for consistent formatting.
        if reference_prefix.is_empty() {
            if self.output.ends_with('\n') {
                self.output.push('\n');
            } else {
                self.output.push('\n');
                self.output.push('\n');
            }
        } else {
            self.ensure_contextual_blank_line_with_prefix(&reference_prefix);
        }
        for (i, styled_lines) in styled_blocks.iter().enumerate() {
            for (line_idx, styled_line) in styled_lines.iter().enumerate() {
                if !reference_prefix.is_empty() {
                    self.output.push_str(&reference_prefix);
                }
                if reference_indent > 0 {
                    self.output.push_str(&" ".repeat(reference_indent));
                }

                self.output.push_str(styled_line);

                if line_idx < styled_lines.len() - 1 || i < styled_blocks.len() - 1 {
                    self.output.push('\n');
                }
            }
        }

        // Add trailing spacing after the link block if requested.
        if add_trailing_newline {
            if reference_prefix.is_empty() {
                self.output.push('\n');
                if in_list {
                    self.output.push('\n');
                }
            } else {
                self.ensure_contextual_blank_line_with_prefix(&reference_prefix);
            }
        }
    }

    pub(super) fn add_paragraph_link_references_with_trailing_newline(
        &mut self,
        add_trailing_newline: bool,
        in_list: bool,
        in_table: bool,
        table_indent: usize,
    ) {
        if self.paragraph_links.is_empty() {
            return;
        }

        let links = std::mem::take(&mut self.paragraph_links);
        self.paragraph_link_counter = 0;
        self.render_link_reference_blocks(
            &links,
            add_trailing_newline,
            in_list,
            in_table,
            table_indent,
        );
    }

    fn build_styled_reference_blocks(
        &self,
        links: &[(String, String)],
        reference_indent: usize,
    ) -> Vec<Vec<String>> {
        let style = create_style(self.theme, ThemeElement::Link);
        let mut styled_blocks: Vec<Vec<String>> = Vec::new();

        for (reference, url) in links {
            let link_line = format!("{} {}", reference, url);

            let wrapped_link = if self.config.is_text_wrapping_enabled() {
                self.wrap_link_line(&link_line, reference_indent)
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

        styled_blocks
    }

    /// Wrap a link line (reference + URL) with proper handling of URL breaking
    pub(super) fn wrap_link_line(&self, link_line: &str, reference_indent: usize) -> String {
        let terminal_width = self.effective_text_width();

        // Link reference lines are printed later with a leading content indentation
        // (self.content_indent spaces). That indentation must be accounted for when
        // deciding how much of the URL can fit on a visual line, otherwise we risk
        // overflowing by 1–N cells and the trailing "..." gets visually clipped to
        // ".." or ".". Compute an effective width for the visible content area.
        let effective_width = terminal_width.saturating_sub(reference_indent);

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

            // Check truncation style - only apply for InlineTable-like modes
            if matches!(
                self.config.link_style,
                LinkStyle::InlineTable | LinkStyle::EndTable
            ) {
                match self.config.link_truncation {
                    LinkTruncationStyle::Cut | LinkTruncationStyle::TableCut => {
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

    pub(super) fn finalize_document_link_references(&mut self) {
        if !matches!(self.config.link_style, LinkStyle::EndTable) {
            return;
        }

        if self.document_links.is_empty() {
            return;
        }

        if self.plaintext_code_block_depth > 0 {
            // Nested plaintext renderers defer formatting to the parent renderer.
            return;
        }

        let styled_blocks =
            self.build_styled_reference_blocks(&self.document_links, self.content_indent);

        if self.output.ends_with('\n') {
            self.output.push('\n');
        } else if !self.output.is_empty() {
            self.output.push('\n');
            self.output.push('\n');
        }

        for (block_idx, styled_lines) in styled_blocks.iter().enumerate() {
            for (line_idx, styled_line) in styled_lines.iter().enumerate() {
                if self.content_indent > 0 {
                    self.output.push_str(&" ".repeat(self.content_indent));
                }

                self.output.push_str(styled_line);

                if line_idx < styled_lines.len() - 1 {
                    self.output.push('\n');
                }
            }

            if block_idx < styled_blocks.len() - 1 {
                self.output.push('\n');
            }
        }

        self.output.push('\n');
        self.document_links.clear();
        self.commit_pending_heading_placeholder_if_content();
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

        let terminal_width = self.effective_text_width();

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
                let prefix = self.current_line_prefix();
                if !prefix.is_empty() {
                    result.push_str(&prefix);
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
        let terminal_width = self.effective_text_width();
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
            let indent = self.current_line_prefix();

            // Replace the space with a newline + indent
            let insert = format!("\n{}", indent);
            let abs_idx = start + space_rel_idx;
            self.output.replace_range(abs_idx..abs_idx + 1, &insert);
        }
    }
}
