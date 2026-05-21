use super::{Event, EventRenderer, Result, Tag, TagEnd};

pub(crate) struct SoftBreakFollowingText {
    text: String,
    ends_paragraph: bool,
}

impl<'a> EventRenderer<'a> {
    pub(super) fn collect_soft_break_following_text(
        events: &[Event<'static>],
    ) -> Option<SoftBreakFollowingText> {
        let mut text = String::new();
        let mut ends_paragraph = false;

        for event in events {
            match event {
                Event::Text(part)
                | Event::Code(part)
                | Event::InlineHtml(part)
                | Event::InlineMath(part) => text.push_str(part.as_ref()),
                Event::FootnoteReference(name) => {
                    text.push_str("[^");
                    text.push_str(name.as_ref());
                    text.push(']');
                }
                Event::SoftBreak
                | Event::HardBreak
                | Event::Html(_)
                | Event::DisplayMath(_)
                | Event::Rule
                | Event::TaskListMarker(_) => break,
                Event::Start(
                    Tag::Emphasis
                    | Tag::Strong
                    | Tag::Strikethrough
                    | Tag::Link { .. }
                    | Tag::Image { .. },
                )
                | Event::End(
                    TagEnd::Emphasis
                    | TagEnd::Strong
                    | TagEnd::Strikethrough
                    | TagEnd::Link
                    | TagEnd::Image,
                ) => {}
                Event::End(TagEnd::Paragraph | TagEnd::Item) => {
                    ends_paragraph = true;
                    break;
                }
                Event::Start(_) | Event::End(_) => break,
            }
        }

        if text.trim().is_empty() {
            None
        } else {
            Some(SoftBreakFollowingText {
                text,
                ends_paragraph,
            })
        }
    }

    pub(super) fn handle_soft_break(
        &mut self,
        next_text: Option<&SoftBreakFollowingText>,
    ) -> Result<()> {
        if self.finalize_pending_callout_label_override() {
            self.suppress_next_soft_break = true;
        }
        if self.suppress_next_soft_break {
            self.suppress_next_soft_break = false;
            return Ok(());
        }

        let collapse = self.should_collapse_soft_break(next_text);
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
        self.current_soft_break_segment_start = self.output.len();

        Ok(())
    }

    fn should_collapse_soft_break(&self, next_text: Option<&SoftBreakFollowingText>) -> bool {
        if !self.config.is_text_wrapping_enabled() {
            return false;
        }

        let Some(next_text) = next_text else {
            return false;
        };
        let next_text_trimmed = next_text.text.trim();
        if next_text_trimmed.is_empty() {
            return false;
        }

        let current_line_clean = if let Some(last_newline) = self.output.rfind('\n') {
            crate::utils::strip_ansi(&self.output[last_newline + 1..])
        } else {
            crate::utils::strip_ansi(&self.output)
        };

        let current_line_width = crate::utils::display_width(&current_line_clean);
        let next_text_width = crate::utils::display_width(next_text_trimmed);
        let needs_separator = current_line_clean
            .chars()
            .next_back()
            .is_some_and(|ch| !ch.is_whitespace());
        let separator_width = usize::from(needs_separator);
        let joined_width = current_line_width + separator_width + next_text_width;
        let effective_width = self.effective_text_width();

        if joined_width > effective_width {
            return self
                .should_collapse_after_short_wrapped_line(current_line_width, effective_width);
        }

        if self.should_preserve_short_final_soft_break(
            next_text,
            next_text_trimmed,
            current_line_width,
            next_text_width,
            joined_width,
            effective_width,
        ) {
            return false;
        }

        true
    }

    fn should_preserve_short_final_soft_break(
        &self,
        next_text: &SoftBreakFollowingText,
        next_text_trimmed: &str,
        current_line_width: usize,
        next_text_width: usize,
        joined_width: usize,
        effective_width: usize,
    ) -> bool {
        if !next_text.ends_paragraph || effective_width == 0 {
            return false;
        }

        let word_count = next_text_trimmed.split_whitespace().count();
        let short_tail_width = (effective_width / 3).max(24);
        let short_final_tail = word_count <= 5 && next_text_width <= short_tail_width;
        let long_single_word_tail = word_count == 1
            && next_text_width >= 12
            && current_line_width * 5 >= effective_width * 3;
        let current_line_is_substantial = current_line_width * 20 >= effective_width * 13;
        let joined_line_is_nearly_full = joined_width * 20 >= effective_width * 19;

        long_single_word_tail
            || (short_final_tail && current_line_is_substantial && joined_line_is_nearly_full)
    }

    fn should_collapse_after_short_wrapped_line(
        &self,
        current_line_width: usize,
        effective_width: usize,
    ) -> bool {
        let segment = self
            .output
            .get(self.current_soft_break_segment_start..)
            .unwrap_or("");
        let segment_wrapped = segment.contains('\n');

        segment_wrapped && current_line_width > 0 && current_line_width * 2 < effective_width
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
