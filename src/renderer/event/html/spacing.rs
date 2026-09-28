use super::*;
use crate::block_spacing::{BlockElement, BlockSpacing};

impl<'a> EventRenderer<'a> {
    pub(super) fn html_block_spacing(
        &self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Option<BlockSpacing> {
        if self.table_state.is_some() {
            return None;
        }
        let block = match element.value().name() {
            "p" => BlockElement::Paragraph,
            "pre" => BlockElement::CodeBlock,
            "blockquote" => BlockElement::Blockquote,
            "dl" => BlockElement::DefinitionList,
            "ul" | "ol" if context.list_depth == 0 => {
                if element.child_elements().any(|child| {
                    child.value().name() == "li" && html_list_item_starts_with_checkbox(&child)
                }) {
                    BlockElement::TaskList
                } else if element.value().name() == "ol" {
                    BlockElement::OrderedList
                } else {
                    BlockElement::UnorderedList
                }
            }
            _ => return None,
        };
        Some(self.config.block_spacing.spacing(block))
    }

    pub(super) fn rebase_html_trailing_spacing(&mut self) {
        let count = self.trailing_blank_line_count();
        if count == 0 {
            return;
        }
        let start = self.output[..self.output.len() - 1]
            .rmatch_indices('\n')
            .nth(count - 1)
            .map_or(0, |(index, _)| index + 1);
        let prefix = self.current_line_prefix();
        let mut replacement = String::new();
        for line in self.output[start..].lines() {
            let (_, source_line) = crate::renderer::line_numbers::strip_internal_markers(line);
            if let Some(line) = source_line {
                replacement.push_str(&crate::renderer::line_numbers::encode_internal_marker(line));
            }
            replacement.push_str(&prefix);
            replacement.push('\n');
        }
        self.output.replace_range(start.., &replacement);
    }
}
