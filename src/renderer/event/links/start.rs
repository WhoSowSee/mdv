use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn handle_link_start(&mut self, dest_url: CowStr) -> Result<()> {
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
            LinkStyle::Clickable | LinkStyle::ClickableForced | LinkStyle::Inline => {
                self.link_counter += 1;
                self.link_references.insert(
                    format!("current_{}", self.link_counter),
                    dest_url.to_string(),
                );
                self.current_link_text.clear();
                self.in_link = true;
            }
            // Hide leaves in_link unset so the link text flows through the normal text path.
            LinkStyle::Hide => {}
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
}
