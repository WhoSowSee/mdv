use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn handle_end_tag(&mut self, tag_end: TagEnd) -> Result<()> {
        match tag_end {
            TagEnd::Paragraph => self.handle_paragraph_end()?,
            TagEnd::Heading(level) => {
                self.handle_header_end(level)?;
            }
            TagEnd::BlockQuote(_) => self.handle_blockquote_end()?,
            TagEnd::CodeBlock => {
                self.handle_code_block_end()?;
                if matches!(self.config.footnote_style, FootnoteStyle::Attached)
                    && !self.current_inline_footnotes.is_empty()
                {
                    self.finalize_inline_footnotes(true, false)?;
                }
            }
            TagEnd::List(_) => self.handle_list_end()?,
            TagEnd::Item => self.handle_item_end()?,
            TagEnd::DefinitionList => self.handle_definition_list_end(),
            TagEnd::DefinitionListDefinition => self.handle_definition_description_end(),
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
                if let Some(ref mut table) = self.table_state
                    && !table.in_header
                {
                    table.rows.push(table.current_row.clone());
                }
            }
            TagEnd::TableCell => {
                self.close_inline_backticks();
                if let Some(ref mut table) = self.table_state {
                    let trimmed_len = table.current_cell.trim_end().len();
                    table.current_cell.truncate(trimmed_len);
                    table
                        .current_row
                        .push(std::mem::take(&mut table.current_cell));
                }
            }
            TagEnd::Link => {
                self.close_inline_backticks();
                self.handle_link_end()?;
            }
            TagEnd::Image => {
                self.handle_image_end()?;
            }
            TagEnd::Emphasis => {
                self.close_inline_backticks();
                self.formatting_stack
                    .retain(|&x| x != ThemeElement::Emphasis);
            }
            TagEnd::Strong => {
                self.close_inline_backticks();
                self.formatting_stack.retain(|&x| x != ThemeElement::Strong);
            }
            TagEnd::Strikethrough => {
                self.close_inline_backticks();
                self.formatting_stack
                    .retain(|&x| x != ThemeElement::Strikethrough);
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) fn handle_hard_break(&mut self) {
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
