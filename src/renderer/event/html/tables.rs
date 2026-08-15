use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn render_html_table(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        let embedded = self.table_state.is_some();
        for caption in element
            .child_elements()
            .filter(|child| child.value().name().eq_ignore_ascii_case("caption"))
        {
            if embedded {
                self.begin_html_table_cell_line(context.list_depth * 2);
                self.render_html_children(caption, context)?;
            } else {
                self.render_html_block(caption, context)?;
            }
        }

        let parent_table = self.table_state.take();
        if !embedded && matches!(self.config.link_style, LinkStyle::InlineTable) {
            self.paragraph_link_counter = 0;
            self.paragraph_links.clear();
        }

        self.table_state = Some(TableState {
            alignments: Vec::new(),
            headers: Vec::new(),
            rows: Vec::new(),
            in_header: true,
            current_row: Vec::new(),
            current_cell: String::new(),
            clickable_link_replacements: Vec::new(),
            inline_url_segments: Vec::new(),
        });

        if let Err(error) = self.render_html_table_section(element, context, false) {
            self.table_state.take();
            self.table_state = parent_table;
            return Err(error);
        }

        let Some(mut table) = self.table_state.take() else {
            self.table_state = parent_table;
            return Ok(());
        };
        normalize_html_table(&mut table);

        if embedded {
            let rendered = self.render_embedded_table(table);
            self.table_state = parent_table;
            let rendered = rendered?;
            if !rendered.is_empty() {
                self.begin_html_table_cell_line(context.list_depth * 2);
                if let Some(ref mut parent) = self.table_state {
                    parent.current_cell.push_str(&rendered);
                }
            }
            return Ok(());
        }

        let table_indent = self.render_table(table)?;

        if matches!(self.config.link_style, LinkStyle::InlineTable)
            && !self.paragraph_links.is_empty()
        {
            self.add_paragraph_link_references_for_table(table_indent);
        }

        Ok(())
    }

    pub(super) fn render_html_table_section(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
        force_header: bool,
    ) -> Result<()> {
        for child in element.child_elements() {
            let name = child.value().name().to_ascii_lowercase();
            match name.as_str() {
                "thead" => self.render_html_table_section(child, context, true)?,
                "tbody" | "tfoot" => self.render_html_table_section(child, context, false)?,
                "tr" => self.render_html_table_row(child, context, force_header)?,
                "caption" | "colgroup" | "col" => {}
                _ => self.render_html_table_section(child, context, force_header)?,
            }
        }

        Ok(())
    }

    pub(super) fn render_html_table_row(
        &mut self,
        row: ElementRef<'_>,
        context: HtmlContext,
        force_header: bool,
    ) -> Result<()> {
        let cells: Vec<_> = row
            .child_elements()
            .filter(|child| {
                let name = child.value().name();
                name.eq_ignore_ascii_case("th") || name.eq_ignore_ascii_case("td")
            })
            .collect();
        if cells.is_empty() {
            return Ok(());
        }

        let has_header_cell = force_header
            || cells
                .iter()
                .any(|cell| cell.value().name().eq_ignore_ascii_case("th"));
        let writes_header = has_header_cell
            && self
                .table_state
                .as_ref()
                .is_some_and(|table| table.headers.is_empty());

        if let Some(ref mut table) = self.table_state {
            table.in_header = writes_header;
            table.current_row.clear();
        }

        let mut alignments = Vec::with_capacity(cells.len());
        for cell in cells {
            let html_alignment = html_alignment(&cell);
            let alignment = html_alignment
                .map(table_alignment_from_html)
                .unwrap_or(Alignment::None);
            let cell_context = html_alignment
                .map(|alignment| context.with_alignment(alignment))
                .unwrap_or(context);
            let content = self.render_html_table_cell(cell, cell_context)?;

            if let Some(ref mut table) = self.table_state {
                table.current_row.push(content);
            }
            if writes_header {
                alignments.push(alignment);
            }
        }

        if let Some(ref mut table) = self.table_state {
            let row = std::mem::take(&mut table.current_row);
            if writes_header {
                table.headers = row;
                table.alignments = alignments;
            } else {
                table.rows.push(row);
            }
            table.current_cell.clear();
        }

        Ok(())
    }

    pub(super) fn render_html_table_cell(
        &mut self,
        cell: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<String> {
        if let Some(ref mut table) = self.table_state {
            table.current_cell.clear();
        }

        self.render_html_children(cell, context)?;

        let content = self
            .table_state
            .as_mut()
            .map(|table| std::mem::take(&mut table.current_cell))
            .unwrap_or_default();

        Ok(content.trim().to_string())
    }
}
