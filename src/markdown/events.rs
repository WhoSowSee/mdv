use super::*;

impl MarkdownProcessor {
    pub(super) fn postprocess_events(
        &self,
        content: &str,
        events: Vec<(Event, Range<usize>)>,
        line_starts: &[usize],
        source_lines: &[Option<usize>],
    ) -> Result<Vec<Event<'static>>> {
        let mut processed = Vec::with_capacity(events.len());
        let mut covered_end = 0usize;

        let mut idx = 0usize;
        while idx < events.len() {
            let current_start = events[idx].1.start;
            self.push_collapsed_blank_source_marker(
                &mut processed,
                content,
                covered_end,
                current_start,
                line_starts,
                source_lines,
            );

            if let Some((Event::Start(Tag::Paragraph), start_range)) = events.get(idx)
                && let (Some((Event::Text(text), _)), Some((Event::End(TagEnd::Paragraph), _))) =
                    (events.get(idx + 1), events.get(idx + 2))
                && text.as_ref().trim() == BLANK_LINE_MARKER
            {
                self.push_event_with_source_marker(
                    &mut processed,
                    Event::Html(BLANK_LINE_MARKER.into()),
                    start_range,
                    line_starts,
                    source_lines,
                );
                covered_end = covered_end.max(
                    events[idx..idx + 3]
                        .iter()
                        .map(|(_, range)| range.end)
                        .max()
                        .unwrap_or(covered_end),
                );
                idx += 3;
                continue;
            }

            if let Some((event, range, end_idx)) =
                raw_html::coalesce_raw_text_container(content, &events, idx)
            {
                self.push_event_with_source_marker(
                    &mut processed,
                    event,
                    &range,
                    line_starts,
                    source_lines,
                );
                covered_end = covered_end.max(range.end);
                idx = end_idx + 1;
                continue;
            }

            if let Some((Event::Start(Tag::CodeBlock(CodeBlockKind::Indented)), start_range)) =
                events.get(idx)
                && let Some(end_idx) = Self::find_code_block_end_index(&events, idx + 1)
                && Self::is_plain_indented_code_block_start(content, start_range.start)
            {
                self.push_demoted_code_block_events(
                    &mut processed,
                    &events[idx + 1..end_idx],
                    line_starts,
                    source_lines,
                )?;
                covered_end = covered_end.max(
                    events[idx..=end_idx]
                        .iter()
                        .map(|(_, range)| range.end)
                        .max()
                        .unwrap_or(covered_end),
                );
                idx = end_idx + 1;
                continue;
            }

            let (event, range) = &events[idx];
            let event = match event {
                Event::Start(tag) => Event::Start(self.convert_tag_to_static(tag.clone())),
                Event::End(tag_end) => Event::End(*tag_end),
                Event::Text(text) => {
                    let processed_text = self.process_text(text);
                    Event::Text(processed_text.to_string().into())
                }
                Event::Code(code) => Event::Code(self.expand_tabs(code.as_ref()).into()),
                other => self.convert_to_static(other.clone()),
            };
            self.push_event_with_source_marker(
                &mut processed,
                event,
                range,
                line_starts,
                source_lines,
            );
            covered_end = covered_end.max(range.end);
            idx += 1;
        }

        Ok(processed)
    }

    pub(super) fn push_collapsed_blank_source_marker(
        &self,
        processed: &mut Vec<Event<'static>>,
        content: &str,
        gap_start: usize,
        gap_end: usize,
        line_starts: &[usize],
        source_lines: &[Option<usize>],
    ) {
        if !self.config.source_line_numbers_enabled()
            || processed.is_empty()
            || gap_end <= gap_start
        {
            return;
        }

        let source_line = line_starts
            .iter()
            .enumerate()
            .find_map(|(line_idx, line_start)| {
                if *line_start < gap_start || *line_start >= gap_end {
                    return None;
                }

                let line_end = line_starts
                    .get(line_idx + 1)
                    .map_or(content.len(), |next_start| next_start.saturating_sub(1));
                content[*line_start..line_end]
                    .trim()
                    .is_empty()
                    .then(|| source_lines.get(line_idx).copied().flatten())
                    .flatten()
            });

        if let Some(source_line) = source_line {
            processed.push(source_lines::blank_event(source_line));
        }
    }

    pub(super) fn find_code_block_end_index(
        events: &[(Event, Range<usize>)],
        start_idx: usize,
    ) -> Option<usize> {
        events
            .iter()
            .enumerate()
            .skip(start_idx)
            .find_map(|(idx, (event, _))| {
                if matches!(event, Event::End(TagEnd::CodeBlock)) {
                    Some(idx)
                } else {
                    None
                }
            })
    }

    pub(super) fn is_plain_indented_code_block_start(content: &str, start_offset: usize) -> bool {
        let safe_start = start_offset.min(content.len());
        let line_start = content[..safe_start].rfind('\n').map_or(0, |idx| idx + 1);
        let indent = &content[line_start..safe_start];

        !indent.is_empty() && indent.chars().all(|ch| ch == ' ' || ch == '\t')
    }

    pub(super) fn push_demoted_code_block_events(
        &self,
        processed: &mut Vec<Event<'static>>,
        events: &[(Event, Range<usize>)],
        outer_line_starts: &[usize],
        outer_source_lines: &[Option<usize>],
    ) -> Result<()> {
        let mut text = String::new();
        for (event, _) in events {
            match event {
                Event::Text(chunk) => text.push_str(chunk.as_ref()),
                Event::SoftBreak | Event::HardBreak => text.push('\n'),
                _ => {}
            }
        }

        let text = text.trim_end_matches('\n');
        if text.is_empty() {
            return Ok(());
        }

        let parser = Parser::new_ext(text, self.options).into_offset_iter();
        let reparsed_events: Vec<(Event, Range<usize>)> = parser.collect();
        let (line_starts, source_lines) = if self.config.source_line_numbers_enabled() {
            let first_processed_line = events
                .first()
                .map(|(_, range)| source_lines::index_for_offset(outer_line_starts, range.start))
                .unwrap_or(0);
            let line_starts = source_lines::starts(text);
            let source_lines = (0..line_starts.len())
                .map(|offset| {
                    outer_source_lines
                        .get(first_processed_line + offset)
                        .copied()
                        .flatten()
                })
                .collect();
            (line_starts, source_lines)
        } else {
            (Vec::new(), Vec::new())
        };
        processed.extend(self.postprocess_events(
            text,
            reparsed_events,
            &line_starts,
            &source_lines,
        )?);
        Ok(())
    }

    pub(super) fn push_event_with_source_marker(
        &self,
        processed: &mut Vec<Event<'static>>,
        event: Event<'static>,
        range: &Range<usize>,
        line_starts: &[usize],
        source_lines: &[Option<usize>],
    ) {
        if !self.config.source_line_numbers_enabled() || !self.event_has_source_content(&event) {
            processed.push(event);
            return;
        }

        let mut line_idx = source_lines::index_for_offset(line_starts, range.start);
        if let Event::Text(ref text) = event
            && text.contains('\n')
        {
            for segment in text.split_inclusive('\n') {
                if let Some(source_line) = source_lines.get(line_idx).copied().flatten() {
                    processed.push(source_lines::event(source_line));
                }
                processed.push(Event::Text(segment.to_string().into()));
                line_idx += segment.bytes().filter(|byte| *byte == b'\n').count();
            }
            return;
        }

        if let Some(source_line) = source_lines.get(line_idx).copied().flatten() {
            processed.push(source_lines::event(source_line));
        }
        processed.push(event);
    }

    pub(super) fn event_has_source_content(&self, event: &Event<'_>) -> bool {
        match event {
            Event::Text(text) => !text.is_empty(),
            Event::Html(html) | Event::InlineHtml(html) => {
                let html = html.as_ref().trim();
                html != BLANK_LINE_MARKER
                    && !(self.config.hide_comments
                        && !self.config.render_html
                        && html.starts_with("<!--")
                        && html.ends_with("-->"))
            }
            Event::Code(_)
            | Event::FootnoteReference(_)
            | Event::TaskListMarker(_)
            | Event::InlineMath(_)
            | Event::DisplayMath(_) => true,
            Event::Start(_) | Event::End(_) | Event::SoftBreak | Event::HardBreak | Event::Rule => {
                false
            }
        }
    }
}
