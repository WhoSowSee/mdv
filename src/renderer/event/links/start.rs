use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn handle_link_start(&mut self, dest_url: CowStr) -> Result<()> {
        if self.pending_callout_label_override {
            self.links.current = Some(CurrentLink::new(LinkDestination::Label));
            return Ok(());
        }
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
                self.links.current = Some(CurrentLink::new(LinkDestination::Url(
                    dest_url.into_string(),
                )));
            }
            LinkStyle::Hide => {}
            LinkStyle::InlineTable => {
                let in_table = self.table_state.is_some();
                if let Some(CalloutState::Active(info)) = self.callout_stack.last_mut() {
                    if in_table {
                        self.links.paragraph_counter += 1;
                        self.links.paragraph.push((
                            format!("[{}]", self.links.paragraph_counter),
                            dest_url.to_string(),
                        ));
                    } else {
                        info.inline_link_counter += 1;
                        let reference = format!("[{}]", info.inline_link_counter);
                        info.inline_links.push((reference, dest_url.to_string()));
                    }
                } else {
                    // Store URL for paragraph-scoped references and start collecting link text
                    self.links.paragraph_counter += 1;
                    self.links.paragraph.push((
                        format!("[{}]", self.links.paragraph_counter),
                        dest_url.to_string(),
                    ));
                }
                self.links.current = Some(CurrentLink::new(LinkDestination::Reference));
            }
            LinkStyle::EndTable => {
                // Store URL for document-scoped references and start collecting link text
                self.links.paragraph_counter += 1;
                self.links.document.push((
                    format!("[{}]", self.links.paragraph_counter),
                    dest_url.to_string(),
                ));
                self.links.current = Some(CurrentLink::new(LinkDestination::Reference));
            }
        }
        Ok(())
    }
}
