use super::{CowStr, EventRenderer, LinkStyle, Result, ThemeElement};

impl<'a> EventRenderer<'a> {
    pub(super) fn handle_text(&mut self, text: CowStr) -> Result<()> {
        if self.in_code_block {
            self.code_block_content.push_str(&text);
        } else if self.in_link {
            match self.config.link_style {
                LinkStyle::Clickable => {
                    // For Clickable mode, collect link text but don't add to output yet
                    // We'll add the complete clickable link in handle_link_end
                    self.current_link_text.push_str(&text);
                    return Ok(());
                }
                LinkStyle::ClickableForced => {
                    // For ClickableForced mode, collect link text but don't add to output yet
                    // We'll add the complete clickable link in handle_link_end
                    self.current_link_text.push_str(&text);
                    return Ok(());
                }
                LinkStyle::Inline => {
                    // For Inline mode, collect link text but don't add to output yet
                    // We'll add the underlined text and URL in handle_link_end with flexible breaking
                    self.current_link_text.push_str(&text);
                    return Ok(());
                }
                LinkStyle::InlineTable => {
                    // Collect link text but don't add to output yet, similar to other modes
                    // We'll add the underlined text and reference number in handle_link_end
                    self.current_link_text.push_str(&text);
                    return Ok(());
                }
                LinkStyle::Hide => {
                    // This shouldn't happen since we don't set in_link for Hide mode anymore
                }
            }
        } else {
            // Process text with wrapping and formatting
            self.process_text_with_wrapping_and_formatting(&text)?;
        }
        if !self.in_code_block && !self.in_link {
            self.commit_pending_heading_placeholder_if_content();
        }
        Ok(())
    }

    /// Process text with wrapping and formatting, handling styled text properly
    fn process_text_with_wrapping_and_formatting(&mut self, text: &str) -> Result<()> {
        // Check if this is for a table cell
        let is_table_cell = self.table_state.is_some();

        if is_table_cell {
            // For table cells, apply formatting directly without complex wrapping
            let formatted_text = self.apply_formatting(text);
            if let Some(ref mut table) = self.table_state {
                table.current_cell.push_str(&formatted_text);
            }
            return Ok(());
        }

        // Add blockquote prefix if we're starting new content in a blockquote
        // Check if we're at the start of a line (after newline or any whitespace-only content)
        if self.blockquote_level > 0 {
            let after_newline = self.output.ends_with('\n');
            let at_start = self.output.is_empty();
            let at_line_start = if let Some(last_newline_pos) = self.output.rfind('\n') {
                // Check if everything after the last newline is just whitespace
                self.output[last_newline_pos + 1..].trim().is_empty()
            } else {
                // No newlines, check if entire output is just whitespace
                self.output.trim().is_empty()
            };

            if after_newline || at_start || at_line_start {
                // Add content indentation first (if we're under a heading)
                if self.content_indent > 0 {
                    self.output.push_str(&" ".repeat(self.content_indent));
                }

                // Then add blockquote prefix
                let prefix = self.render_blockquote_prefix();
                self.output.push_str(&prefix);
            }
        }

        // Check if we need to wrap text. When no explicit cols are provided,
        // wrap to the detected terminal width (unless --no-wrap is set).
        let should_wrap = self.config.is_text_wrapping_enabled();

        if should_wrap && !self.formatting_stack.is_empty() {
            // For styled text, prefer continuous decoration for strike-through
            if self.formatting_stack.contains(&ThemeElement::Strikethrough) {
                self.process_strikethrough_text_with_wrapping(text)?;
            } else {
                // Default styled processing (per-unit formatting)
                self.process_styled_text_with_wrapping(text)?;
            }
        } else {
            // Regular text processing
            self.process_regular_text(text, should_wrap)?;
        }

        Ok(())
    }

    /// Process styled text with proper character/word-level wrapping like the original logic
    fn process_styled_text_with_wrapping(&mut self, text: &str) -> Result<()> {
        let terminal_width = self.config.get_terminal_width();

        // The effective width is the full terminal width since current_line_width
        // already includes any indentation that's been added to the current line
        let effective_width = terminal_width;

        // Determine wrap mode based on config
        let wrap_mode = self.config.text_wrap_mode();

        // Split text into wrappable units (words or characters) while preserving formatting
        let units = match wrap_mode {
            crate::utils::WrapMode::Word => self.split_text_into_words_styled(text),
            crate::utils::WrapMode::Character => self.split_text_into_characters_styled(text),
            crate::utils::WrapMode::None => vec![text.to_string()],
        };

        // Process each unit individually with formatting
        for (i, unit) in units.iter().enumerate() {
            if unit.trim().is_empty() && i > 0 {
                // Handle whitespace between units
                self.output.push_str(unit);
                continue;
            }

            // Check if adding this unit would exceed line width
            let current_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
                crate::utils::strip_ansi(&self.output[last_newline + 1..])
            } else {
                crate::utils::strip_ansi(&self.output)
            };

            let current_line_width = crate::utils::display_width(&current_line_clean);
            let unit_width = crate::utils::display_width(unit);

            // For InlineTable links, account for the reference number that will be added
            let additional_width =
                if self.in_link && matches!(self.config.link_style, LinkStyle::InlineTable) {
                    // Calculate the width of the reference number like [1], [2], etc.
                    let ref_num_str = format!("[{}]", self.paragraph_link_counter);
                    crate::utils::display_width(&ref_num_str)
                } else {
                    0
                };

            let would_exceed = current_line_width + unit_width + additional_width > effective_width;

            // Force line break if needed (but not for the first unit on a line)
            if would_exceed && current_line_width > 0 && !current_line_clean.trim().is_empty() {
                // Check if we should break before this unit
                let should_break = match wrap_mode {
                    crate::utils::WrapMode::Word => {
                        // For word wrapping, break before words (but not before punctuation)
                        !unit.trim_start().starts_with(',')
                            && !unit.trim_start().starts_with('.')
                            && !unit.trim_start().starts_with(';')
                            && !unit.trim_start().starts_with(':')
                            && !unit.trim_start().starts_with('!')
                            && !unit.trim_start().starts_with('?')
                            && !unit.trim_start().starts_with(')')
                            && !unit.trim_start().starts_with(']')
                            && !unit.trim_start().starts_with('}')
                    }
                    crate::utils::WrapMode::Character => true, // Always break for character mode
                    crate::utils::WrapMode::None => false,
                };

                if should_break {
                    // Centralized handler adds correct indent for lists, blockquotes, headings
                    self.push_newline_with_context();
                }
            }

            // Apply formatting and add to output
            let formatted_unit = self.apply_formatting(unit);

            // Add content indentation for new lines if needed
            // But don't add it if we're continuing text on the same line (like after inline links)
            let should_add_indent = (self.output.ends_with('\n') || self.output.is_empty())
                && !formatted_unit.trim().is_empty();

            // Check if we're immediately after content that shouldn't get extra indentation
            let after_inline_content = if let Some(last_newline) = self.output.rfind('\n') {
                let line_content = &self.output[last_newline + 1..];
                // If the line has content (not just whitespace), we're continuing on the same line
                !line_content.trim().is_empty()
            } else {
                // No newlines, check if we have any content
                !self.output.trim().is_empty()
            };

            // Don't add indentation if we're continuing on the same line OR
            // if we just processed a link (which may have wrapped URLs)
            if should_add_indent && !after_inline_content {
                self.push_indent_for_line_start();
            }

            self.output.push_str(&formatted_unit);
        }

        Ok(())
    }

    /// Split text into words for word-based wrapping (for styled text)
    fn split_text_into_words_styled(&self, text: &str) -> Vec<String> {
        let mut words = Vec::new();
        let mut current_word = String::new();
        let mut in_whitespace = false;

        for ch in text.chars() {
            if ch.is_whitespace() {
                if !in_whitespace && !current_word.is_empty() {
                    words.push(current_word.clone());
                    current_word.clear();
                }
                current_word.push(ch);
                in_whitespace = true;
            } else {
                if in_whitespace && !current_word.is_empty() {
                    words.push(current_word.clone());
                    current_word.clear();
                }
                current_word.push(ch);
                in_whitespace = false;
            }
        }

        if !current_word.is_empty() {
            words.push(current_word);
        }

        words
    }

    /// Split text into characters for character-based wrapping (for styled text)
    fn split_text_into_characters_styled(&self, text: &str) -> Vec<String> {
        text.chars().map(|c| c.to_string()).collect()
    }

    /// Calculate proper indentation for list content continuation lines
    pub(super) fn calculate_list_content_indent(&self) -> usize {
        let mut total_indent = 0;

        // Add heading content indentation
        total_indent += self.content_indent;

        // Add list nesting indentation (2 spaces per level)
        let indent_level = self.list_stack.len().saturating_sub(1);
        total_indent += indent_level * 2;

        // Add space for the list marker
        if let Some(list_state) = self.list_stack.last() {
            let marker_width = if list_state.is_ordered {
                // For ordered lists: "1. ", "2. ", etc. - typically 3 characters
                3
            } else {
                // For unordered lists: "- " - 2 characters
                2
            };
            total_indent += marker_width;
        }

        total_indent
    }

    /// Process text with underline formatting applied to continuous fragments between line breaks
    pub(super) fn process_underlined_text_with_wrapping(&mut self, text: &str) -> Result<()> {
        let should_wrap = self.config.is_text_wrapping_enabled();

        if !should_wrap {
            // No wrapping - just apply underline to entire text
            let formatted_text = if !self.config.no_colors {
                format!("\x1b[4m{}\x1b[0m", text)
            } else {
                text.to_string()
            };
            self.output.push_str(&formatted_text);
            return Ok(());
        }

        let terminal_width = self.config.get_terminal_width();
        let effective_width = terminal_width;

        // Determine wrap mode based on config
        let wrap_mode = self.config.text_wrap_mode();

        // Split text into wrappable units (words or characters)
        let units = match wrap_mode {
            crate::utils::WrapMode::Word => self.split_text_into_words_styled(text),
            crate::utils::WrapMode::Character => self.split_text_into_characters_styled(text),
            crate::utils::WrapMode::None => vec![text.to_string()],
        };

        // Process units in groups - each group becomes one continuous underlined fragment
        let mut current_fragment = String::new();

        // Get initial line width
        let initial_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
            crate::utils::strip_ansi(&self.output[last_newline + 1..])
        } else {
            crate::utils::strip_ansi(&self.output)
        };
        let mut fragment_start_line_width = crate::utils::display_width(&initial_line_clean);

        // If there's no space left on the current line, move to a new one before adding any underlined text
        // If only 0 or 1 cells remain, start on a fresh line to avoid placing
        // a single dangling character at the line edge (which looks like overflow).
        if effective_width.saturating_sub(fragment_start_line_width) <= 1 && !text.trim().is_empty()
        {
            self.push_newline_with_context();

            // Account for full visual prefix on the new line (heading indent, list content
            // indent, blockquote pipes, etc.)
            fragment_start_line_width = self.compute_line_start_context_width();
        }

        for (i, unit) in units.iter().enumerate() {
            let is_ws = unit.trim().is_empty();
            let unit_width = crate::utils::display_width(unit);
            let current_fragment_width = crate::utils::display_width(&current_fragment);
            let would_exceed =
                fragment_start_line_width + current_fragment_width + unit_width > effective_width;

            // Special handling for whitespace: never allow trailing spaces to cause overflow
            if is_ws && i > 0 {
                if would_exceed && !current_fragment.trim().is_empty() {
                    // Flush current fragment and break line; drop the whitespace (no leading spaces)
                    let fragment_to_format = current_fragment.trim_end();
                    let trailing_spaces = &current_fragment[fragment_to_format.len()..];

                    let formatted_fragment = if !self.config.no_colors {
                        format!("\x1b[4m{}\x1b[0m{}", fragment_to_format, trailing_spaces)
                    } else {
                        current_fragment.clone()
                    };
                    self.output.push_str(&formatted_fragment);

                    // Start new visual line with proper indent/prefix
                    self.push_newline_with_context();

                    fragment_start_line_width = self.compute_line_start_context_width();

                    current_fragment.clear();
                    continue; // Skip adding whitespace at the start of the new line
                } else {
                    // Safe to keep whitespace in the fragment
                    current_fragment.push_str(unit);
                    continue;
                }
            }

            if would_exceed && !current_fragment.trim().is_empty() {
                // We need to break - output current fragment first
                // Remove trailing spaces before applying underline to avoid underlined spaces at line end
                let fragment_to_format = current_fragment.trim_end();
                let trailing_spaces = &current_fragment[fragment_to_format.len()..];

                let formatted_fragment = if !self.config.no_colors {
                    format!("\x1b[4m{}\x1b[0m{}", fragment_to_format, trailing_spaces)
                } else {
                    current_fragment.clone()
                };
                self.output.push_str(&formatted_fragment);

                // Check if we should break before this unit
                let should_break = match wrap_mode {
                    crate::utils::WrapMode::Word => {
                        // For word wrapping, break before words (but not before punctuation)
                        !unit.trim_start().starts_with(',')
                            && !unit.trim_start().starts_with('.')
                            && !unit.trim_start().starts_with(';')
                            && !unit.trim_start().starts_with(':')
                            && !unit.trim_start().starts_with('!')
                            && !unit.trim_start().starts_with('?')
                            && !unit.trim_start().starts_with(')')
                            && !unit.trim_start().starts_with(']')
                            && !unit.trim_start().starts_with('}')
                    }
                    crate::utils::WrapMode::Character => true, // Always break for character mode
                    crate::utils::WrapMode::None => false,
                };

                if should_break {
                    self.push_newline_with_context();

                    // Reset fragment tracking for new visual line
                    fragment_start_line_width = self.compute_line_start_context_width();
                }

                // Start new fragment with current unit
                current_fragment = unit.clone();
            } else {
                if would_exceed {
                    // Nothing in fragment yet but even this unit would exceed the line.
                    // Break the line first, then start with this unit.
                    self.push_newline_with_context();

                    fragment_start_line_width = self.compute_line_start_context_width();
                }

                // Add unit to current fragment
                current_fragment.push_str(unit);
            }
        }

        // Output remaining fragment if any
        if !current_fragment.is_empty() {
            // Remove trailing spaces before applying underline to avoid underlined spaces at line end
            let fragment_to_format = current_fragment.trim_end();
            let trailing_spaces = &current_fragment[fragment_to_format.len()..];

            let formatted_fragment = if !self.config.no_colors {
                format!("\x1b[4m{}\x1b[0m{}", fragment_to_format, trailing_spaces)
            } else {
                current_fragment
            };
            self.output.push_str(&formatted_fragment);
        }

        Ok(())
    }

    /// Process text with strikethrough formatting applied as a continuous run (includes spaces)
    fn process_strikethrough_text_with_wrapping(&mut self, text: &str) -> Result<()> {
        let should_wrap = self.config.is_text_wrapping_enabled();

        if !should_wrap {
            // No wrapping - apply full formatting (including strikethrough) to entire text
            let formatted_text = self.apply_formatting(text);
            self.output.push_str(&formatted_text);
            return Ok(());
        }

        let terminal_width = self.config.get_terminal_width();
        let effective_width = terminal_width;

        // Determine wrap mode based on config
        let wrap_mode = self.config.text_wrap_mode();

        // Split text into wrappable units (words or characters)
        let units = match wrap_mode {
            crate::utils::WrapMode::Word => self.split_text_into_words_styled(text),
            crate::utils::WrapMode::Character => self.split_text_into_characters_styled(text),
            crate::utils::WrapMode::None => vec![text.to_string()],
        };

        // Process units in groups - each group becomes one continuous struck fragment
        let mut current_fragment = String::new();

        // Initial line width (without ANSI)
        let initial_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
            crate::utils::strip_ansi(&self.output[last_newline + 1..])
        } else {
            crate::utils::strip_ansi(&self.output)
        };
        let mut fragment_start_line_width = crate::utils::display_width(&initial_line_clean);

        // If little space left on the current line, move to a new one before adding any struck text
        if effective_width.saturating_sub(fragment_start_line_width) <= 1 && !text.trim().is_empty()
        {
            self.push_newline_with_context();
            fragment_start_line_width = self.compute_line_start_context_width();
        }

        for (i, unit) in units.iter().enumerate() {
            let is_ws = unit.trim().is_empty();
            let unit_width = crate::utils::display_width(unit);
            let current_fragment_width = crate::utils::display_width(&current_fragment);
            let would_exceed =
                fragment_start_line_width + current_fragment_width + unit_width > effective_width;

            // Whitespace handling: keep inside fragment unless it would overflow the line
            if is_ws && i > 0 {
                if would_exceed && !current_fragment.trim().is_empty() {
                    // Flush current fragment and break line; drop whitespace at new line start
                    let fragment_to_format = current_fragment.trim_end();
                    let trailing_spaces = &current_fragment[fragment_to_format.len()..];

                    // Apply full formatting (includes strike) to the non-trailing part, then append spaces
                    let formatted_fragment = format!(
                        "{}{}",
                        self.apply_formatting(fragment_to_format),
                        trailing_spaces
                    );
                    self.output.push_str(&formatted_fragment);

                    // Start new visual line with correct context indentation
                    self.push_newline_with_context();
                    fragment_start_line_width = self.compute_line_start_context_width();

                    current_fragment.clear();
                    continue;
                } else {
                    current_fragment.push_str(unit);
                    continue;
                }
            }

            if would_exceed && !current_fragment.trim().is_empty() {
                // Break: output current fragment first
                let fragment_to_format = current_fragment.trim_end();
                let trailing_spaces = &current_fragment[fragment_to_format.len()..];
                let formatted_fragment = format!(
                    "{}{}",
                    self.apply_formatting(fragment_to_format),
                    trailing_spaces
                );
                self.output.push_str(&formatted_fragment);

                // Decide if we break before this unit (word wrap rules)
                let should_break = match wrap_mode {
                    crate::utils::WrapMode::Word => {
                        !unit.trim_start().starts_with(',')
                            && !unit.trim_start().starts_with('.')
                            && !unit.trim_start().starts_with(';')
                            && !unit.trim_start().starts_with(':')
                            && !unit.trim_start().starts_with('!')
                            && !unit.trim_start().starts_with('?')
                            && !unit.trim_start().starts_with(')')
                            && !unit.trim_start().starts_with(']')
                            && !unit.trim_start().starts_with('}')
                    }
                    crate::utils::WrapMode::Character => true,
                    crate::utils::WrapMode::None => false,
                };

                if should_break {
                    self.push_newline_with_context();
                    fragment_start_line_width = self.compute_line_start_context_width();
                }

                current_fragment = unit.clone();
            } else {
                if would_exceed {
                    // Nothing in fragment yet, but unit would exceed -> break line first
                    self.push_newline_with_context();
                    fragment_start_line_width = self.compute_line_start_context_width();
                }

                current_fragment.push_str(unit);
            }
        }

        // Output remaining fragment if any
        if !current_fragment.is_empty() {
            let fragment_to_format = current_fragment.trim_end();
            let trailing_spaces = &current_fragment[fragment_to_format.len()..];
            let formatted_fragment = format!(
                "{}{}",
                self.apply_formatting(fragment_to_format),
                trailing_spaces
            );
            self.output.push_str(&formatted_fragment);
        }

        Ok(())
    }
    fn process_regular_text(&mut self, text: &str, should_wrap: bool) -> Result<()> {
        // Use the same word-by-word logic as styled text for consistent behavior
        if should_wrap {
            let terminal_width = self.config.get_terminal_width();

            // Use full terminal width as effective width since current_line_width already includes indents
            let effective_width = terminal_width;

            // Determine wrap mode based on config
            let wrap_mode = self.config.text_wrap_mode();

            // Split text into wrappable units (words or characters)
            let units = match wrap_mode {
                crate::utils::WrapMode::Word => self.split_text_into_words_styled(text),
                crate::utils::WrapMode::Character => self.split_text_into_characters_styled(text),
                crate::utils::WrapMode::None => vec![text.to_string()],
            };

            // Process each unit individually
            for (_i, unit) in units.iter().enumerate() {
                if unit.trim().is_empty() {
                    // Handle whitespace cautiously: don't let a trailing space overflow the line
                    let current_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
                        crate::utils::strip_ansi(&self.output[last_newline + 1..])
                    } else {
                        crate::utils::strip_ansi(&self.output)
                    };
                    let current_line_width = crate::utils::display_width(&current_line_clean);
                    let space_width = crate::utils::display_width(unit);
                    if current_line_width + space_width > effective_width {
                        // Break visual line and skip adding whitespace at start of next line
                        self.push_newline_with_context();
                    } else {
                        self.output.push_str(unit);
                    }
                    continue;
                }

                // Check if adding this unit would exceed line width
                let current_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
                    crate::utils::strip_ansi(&self.output[last_newline + 1..])
                } else {
                    crate::utils::strip_ansi(&self.output)
                };

                let current_line_width = crate::utils::display_width(&current_line_clean);
                let unit_width = crate::utils::display_width(unit);

                // For InlineTable links, account for the reference number that will be added
                let additional_width =
                    if self.in_link && matches!(self.config.link_style, LinkStyle::InlineTable) {
                        // Calculate the width of the reference number like [1], [2], etc.
                        let ref_num_str = format!("[{}]", self.paragraph_link_counter);
                        crate::utils::display_width(&ref_num_str)
                    } else {
                        0
                    };

                let would_exceed =
                    current_line_width + unit_width + additional_width > effective_width;

                // Force line break if needed (but not for the first unit on a line)
                if would_exceed && current_line_width > 0 && !current_line_clean.trim().is_empty() {
                    // Check if we should break before this unit
                    let should_break = match wrap_mode {
                        crate::utils::WrapMode::Word => {
                            // For word wrapping, break before words (but not before punctuation)
                            !unit.trim_start().starts_with(',')
                                && !unit.trim_start().starts_with('.')
                                && !unit.trim_start().starts_with(';')
                                && !unit.trim_start().starts_with(':')
                                && !unit.trim_start().starts_with('!')
                                && !unit.trim_start().starts_with('?')
                                && !unit.trim_start().starts_with(')')
                                && !unit.trim_start().starts_with(']')
                                && !unit.trim_start().starts_with('}')
                        }
                        crate::utils::WrapMode::Character => true, // Always break for character mode
                        crate::utils::WrapMode::None => false,
                    };

                    if should_break {
                        self.push_newline_with_context();
                    }
                }

                // Apply formatting (no-op for regular text) and add to output
                let formatted_unit = self.apply_formatting(unit);

                // Add content indentation for new lines if needed
                // But don't add it if we're continuing text on the same line (like after inline links)
                let should_add_indent = (self.output.ends_with('\n') || self.output.is_empty())
                    && !formatted_unit.trim().is_empty();

                // Check if we're immediately after content that shouldn't get extra indentation
                let after_inline_content = if let Some(last_newline) = self.output.rfind('\n') {
                    let line_content = &self.output[last_newline + 1..];
                    // If the line has content (not just whitespace), we're continuing on the same line
                    !line_content.trim().is_empty()
                } else {
                    // No newlines, check if we have any content
                    !self.output.trim().is_empty()
                };

                if should_add_indent && !after_inline_content {
                    self.push_indent_for_line_start();
                }

                self.output.push_str(&formatted_unit);
            }
        } else {
            // No wrapping - still ensure correct indentation at visual line starts
            let final_text = self.apply_formatting(text);

            // Add content indentation for new visual lines when appropriate
            if (self.output.ends_with('\n') || self.output.is_empty())
                && !final_text.trim().is_empty()
            {
                // If the current line (after the last newline) already contains
                // non-whitespace content, we are continuing on the same line and
                // must not add extra indentation.
                let after_inline_content = if let Some(last_newline) = self.output.rfind('\n') {
                    let line_content = &self.output[last_newline + 1..];
                    !line_content.trim().is_empty()
                } else {
                    !self.output.trim().is_empty()
                };

                if !after_inline_content {
                    self.push_indent_for_line_start();
                }
            }

            self.output.push_str(&final_text);
        }

        Ok(())
    }
}
