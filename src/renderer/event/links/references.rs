use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn add_paragraph_link_references(&mut self) {
        self.add_paragraph_link_references_with_trailing_newline(true, false, 0);
    }

    pub(in crate::renderer::event) fn add_paragraph_link_references_for_table(
        &mut self,
        table_indent: usize,
    ) {
        self.add_paragraph_link_references_with_trailing_newline(true, true, table_indent);
    }

    pub(in crate::renderer::event) fn render_link_reference_blocks(
        &mut self,
        links: &[(String, String)],
        add_trailing_newline: bool,
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

        let spacing = self
            .config
            .block_spacing
            .spacing(BlockElement::InlineReferences);
        self.ensure_contextual_blank_lines_with_prefix(spacing.top, &reference_prefix);
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
            self.ensure_contextual_blank_lines_with_prefix(spacing.bottom, &reference_prefix);
        }
    }

    pub(in crate::renderer::event) fn add_paragraph_link_references_with_trailing_newline(
        &mut self,
        add_trailing_newline: bool,
        in_table: bool,
        table_indent: usize,
    ) {
        if self.paragraph_links.is_empty() {
            return;
        }

        let links = std::mem::take(&mut self.paragraph_links);
        self.paragraph_link_counter = 0;
        self.render_link_reference_blocks(&links, add_trailing_newline, in_table, table_indent);
    }

    pub(super) fn build_styled_reference_blocks(
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
    pub(in crate::renderer::event) fn wrap_link_line(
        &self,
        link_line: &str,
        reference_indent: usize,
    ) -> String {
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

    pub(in crate::renderer::event) fn finalize_document_link_references(&mut self) {
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

        let spacing = self
            .config
            .block_spacing
            .spacing(BlockElement::EndReferences);
        self.ensure_contextual_blank_lines(spacing.top);

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

        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        self.ensure_contextual_blank_lines(spacing.bottom);
        self.document_links.clear();
        self.commit_pending_heading_placeholder_if_content();
    }
}
