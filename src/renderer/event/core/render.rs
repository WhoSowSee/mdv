use super::*;

impl<'a> EventRenderer<'a> {
    pub(crate) fn render_events(&mut self, events: Vec<Event<'static>>) -> Result<String> {
        let (events, mut definitions) = self.extract_footnote_definitions(events);

        if !self.footnote_definitions.is_empty() {
            for existing in self.footnote_definitions.iter() {
                if !definitions.iter().any(|def| def.name == existing.name) {
                    definitions.push(existing.clone());
                }
            }
        }
        self.footnote_definitions = definitions;
        self.prepare_block_spacing_elements(&events);

        if matches!(self.config.heading_layout, crate::cli::HeadingLayout::Level)
            && self.config.smart_indent
        {
            self.prepare_smart_heading_indents(&events);
        } else {
            self.smart_level_indents.clear();
        }

        for event in events {
            self.process_event(event)?;
        }

        self.close_inline_backticks();
        self.flush_pending_html_block_buffer()?;
        self.finalize_pending_heading_placeholder();
        if matches!(self.config.footnote_style, FootnoteStyle::Attached)
            && !self.current_inline_footnotes.is_empty()
        {
            self.finalize_inline_footnotes(true, false)?;
        }
        self.finalize_document_link_references();
        self.finalize_document_footnotes()?;

        // Remove excessive trailing newlines, but keep one
        let mut result = self.output.trim_end().to_string();
        if !result.is_empty() {
            result.push('\n');
        }

        Ok(result)
    }

    pub(super) fn prepare_smart_heading_indents(&mut self, events: &[Event]) {
        self.smart_level_indents.clear();
        self.prepared_blockquote_smart_indents.clear();

        let mut present = [false; 6];
        struct BlockquoteScanFrame {
            start_index: usize,
            present: [bool; 6],
        }

        let mut blockquote_stack: Vec<BlockquoteScanFrame> = Vec::new();
        let mut blockquote_maps: Vec<Option<HashMap<HeadingLevel, usize>>> =
            vec![None; events.len()];

        for (idx, event) in events.iter().enumerate() {
            match event {
                Event::Start(Tag::BlockQuote(_)) => {
                    blockquote_stack.push(BlockquoteScanFrame {
                        start_index: idx,
                        present: [false; 6],
                    });
                }
                Event::End(TagEnd::BlockQuote(_)) => {
                    if let Some(frame) = blockquote_stack.pop() {
                        let map = Self::build_smart_indent_map(&frame.present);
                        blockquote_maps[frame.start_index] = Some(map);
                    }
                }
                Event::Start(Tag::Heading { level, .. }) => {
                    let idx = Self::heading_level_to_number(*level) - 1;
                    if blockquote_stack.is_empty() {
                        present[idx] = true;
                    } else {
                        for frame in blockquote_stack.iter_mut() {
                            frame.present[idx] = true;
                        }
                    }
                }
                _ => {}
            }
        }

        self.smart_level_indents = Self::build_smart_indent_map(&present);
        for entry in blockquote_maps.into_iter().flatten() {
            self.prepared_blockquote_smart_indents.push_back(entry);
        }
    }

    pub(super) fn build_smart_indent_map(present: &[bool; 6]) -> HashMap<HeadingLevel, usize> {
        let mut map = HashMap::new();
        let min_idx = match present.iter().position(|&is_present| is_present) {
            Some(idx) => idx,
            None => return map,
        };

        for (idx, is_present) in present.iter().enumerate() {
            if !is_present {
                continue;
            }

            let missing_between = (min_idx + 1..idx)
                .filter(|&gap_idx| !present[gap_idx])
                .count();

            let planned_indent = idx.saturating_sub(missing_between).saturating_sub(min_idx);

            if let Some(level) = Self::number_to_heading_level(idx + 1) {
                map.insert(level, planned_indent);
            }
        }

        map
    }

    pub(super) fn heading_level_to_number(level: HeadingLevel) -> usize {
        match level {
            HeadingLevel::H1 => 1,
            HeadingLevel::H2 => 2,
            HeadingLevel::H3 => 3,
            HeadingLevel::H4 => 4,
            HeadingLevel::H5 => 5,
            HeadingLevel::H6 => 6,
        }
    }

    pub(super) fn number_to_heading_level(number: usize) -> Option<HeadingLevel> {
        match number {
            1 => Some(HeadingLevel::H1),
            2 => Some(HeadingLevel::H2),
            3 => Some(HeadingLevel::H3),
            4 => Some(HeadingLevel::H4),
            5 => Some(HeadingLevel::H5),
            6 => Some(HeadingLevel::H6),
            _ => None,
        }
    }
}
