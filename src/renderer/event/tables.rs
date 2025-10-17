use super::{EventRenderer, LinkStyle, Result, TableRenderer, TableState};
use crate::utils::strip_ansi;
use pulldown_cmark::Alignment;

impl<'a> EventRenderer<'a> {
    pub(super) fn handle_table_end(&mut self) -> Result<()> {
        if let Some(table) = self.table_state.take() {
            self.render_table(table)?;
        }

        // Add accumulated link references for InlineTable mode at the end of the table
        if matches!(self.config.link_style, LinkStyle::InlineTable)
            && !self.paragraph_links.is_empty()
        {
            self.add_paragraph_link_references_for_table();
        }

        Ok(())
    }

    pub(super) fn render_table(&mut self, mut table: TableState) -> Result<()> {
        let headers_empty = table
            .headers
            .iter()
            .all(|header| strip_ansi(header).trim().is_empty());
        let rows_empty = table
            .rows
            .iter()
            .all(|row| row.iter().all(|cell| strip_ansi(cell).trim().is_empty()));

        if table.headers.is_empty() && !self.config.show_empty_elements {
            return Ok(());
        }

        if headers_empty && rows_empty && !self.config.show_empty_elements {
            return Ok(());
        }

        if self.config.show_empty_elements {
            if table.headers.is_empty() {
                table.headers.push(" ".to_string());
            } else if headers_empty {
                for header in table.headers.iter_mut() {
                    if strip_ansi(header).trim().is_empty() {
                        *header = " ".to_string();
                    }
                }
            }

            if table.alignments.len() < table.headers.len() {
                table.alignments.extend(
                    std::iter::repeat(Alignment::Left)
                        .take(table.headers.len().saturating_sub(table.alignments.len())),
                );
            }

            if rows_empty {
                if table.rows.is_empty() {
                    table
                        .rows
                        .push(vec![" ".to_string(); table.headers.len().max(1)]);
                } else {
                    for row in table.rows.iter_mut() {
                        if row.len() < table.headers.len() {
                            row.extend(
                                std::iter::repeat(String::new())
                                    .take(table.headers.len().saturating_sub(row.len())),
                            );
                        }
                        for cell in row.iter_mut() {
                            if strip_ansi(cell).trim().is_empty() {
                                *cell = " ".to_string();
                            }
                        }
                    }
                }
            }
        } else if table.headers.is_empty() {
            return Ok(());
        }

        let terminal_width = self.config.get_terminal_width();
        let table_renderer = TableRenderer::new(
            self.theme,
            self.config.no_colors,
            terminal_width,
            self.config.table_wrap,
        );

        let mut rendered_table =
            table_renderer.render_table(&table.headers, &table.rows, &table.alignments)?;

        if !table.inline_references.is_empty() {
            rendered_table = crate::table::apply_inline_reference_styles(
                rendered_table,
                &table.inline_references,
                self.config.no_colors,
            );
        }

        self.ensure_contextual_blank_line();

        self.output.push_str(&rendered_table);
        self.output.push('\n');
        self.commit_pending_heading_placeholder_if_content();

        Ok(())
    }
}
