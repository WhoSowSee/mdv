use super::{EventRenderer, Result};

impl<'a> EventRenderer<'a> {
    pub(super) fn handle_soft_break(&mut self) -> Result<()> {
        if self.finalize_pending_callout_label_override() {
            self.suppress_next_soft_break = true;
        }
        if self.suppress_next_soft_break {
            self.suppress_next_soft_break = false;
            return Ok(());
        }

        if self.pending_task_marker && self.is_custom_task_marker(&self.pending_task_marker_buffer)
        {
            self.pending_task_marker_buffer.push(' ');
        }

        let collapse = self.config.reflow && self.config.is_text_wrapping_enabled();
        if self.in_link {
            self.current_link_text
                .push(if collapse { ' ' } else { '\n' });
        } else if let Some(ref mut table) = self.table_state {
            table.current_cell.push(if collapse { ' ' } else { '\n' });
        } else if collapse {
            self.push_collapsed_soft_break_space();
        } else {
            self.output.push('\n');
        }
        Ok(())
    }

    fn push_collapsed_soft_break_space(&mut self) {
        let last_char_is_whitespace = self
            .output
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace);

        if !self.output.is_empty() && !last_char_is_whitespace {
            self.output.push(' ');
        }
    }
}
