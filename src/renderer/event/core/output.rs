use super::EventRenderer;
use std::ops::Range;

impl EventRenderer<'_> {
    pub(in crate::renderer::event) fn replace_output_spacing(
        &mut self,
        range: Range<usize>,
        replacement: &str,
    ) {
        self.output.replace_range(range.clone(), replacement);
        let update = |position: &mut usize| {
            if *position > range.start {
                // Positions inside removed spacing stay before the replacement.
                *position = if *position < range.end {
                    range.start
                } else {
                    range.start + replacement.len() + (*position - range.end)
                };
            }
        };

        for position in [
            &mut self.current_paragraph_start,
            &mut self.current_heading_start,
        ]
        .into_iter()
        .flatten()
        {
            update(position);
        }
        if let Some((start, _)) = &mut self.pending_heading_placeholder {
            update(start);
        }
        for start in &mut self.blockquote_starts {
            update(start);
        }
        for list in &mut self.list_stack {
            update(&mut list.block_start);
            for position in [
                &mut list.current_item_start,
                &mut list.current_item_marker_start,
                &mut list.current_item_marker_end,
            ]
            .into_iter()
            .flatten()
            {
                update(position);
            }
        }
        for list in &mut self.definition_list_stack {
            list.update_output_positions(update);
        }
    }
}
