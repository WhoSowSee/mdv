use super::{ElementRef, EventRenderer, HtmlContext, Result, ThemeElement};

impl<'a> EventRenderer<'a> {
    pub(super) fn render_html_input(&mut self, element: ElementRef<'_>) -> Result<()> {
        let input_type = element
            .attr("type")
            .map(str::trim)
            .filter(|kind| !kind.is_empty())
            .unwrap_or("text")
            .to_ascii_lowercase();

        match input_type.as_str() {
            "hidden" => Ok(()),
            "checkbox" => {
                let marker = self.checkbox_marker(element.attr("checked").is_some());
                if let Some(ref mut table) = self.table_state {
                    table.current_cell.push_str(&marker);
                    table.current_cell.push(' ');
                } else {
                    self.note_paragraph_content();
                    self.output.push_str(&marker);
                    self.output.push(' ');
                }
                Ok(())
            }
            "radio" => {
                let marker = if element.attr("checked").is_some() {
                    "(●)"
                } else {
                    "( )"
                };
                self.render_html_control_literal(marker)
            }
            "submit" => {
                let value = element.attr("value").unwrap_or("Submit");
                self.render_html_control_literal(&format!("[{value}]"))
            }
            "reset" => {
                let value = element.attr("value").unwrap_or("Reset");
                self.render_html_control_literal(&format!("[{value}]"))
            }
            "button" => {
                let value = element.attr("value").unwrap_or_default();
                self.render_html_control_literal(&format!("[{value}]"))
            }
            _ => {
                let value = element
                    .attr("value")
                    .or_else(|| element.attr("placeholder"))
                    .unwrap_or_default();
                self.render_html_control_literal(&format!("[{value}]"))
            }
        }
    }

    pub(super) fn render_html_button(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        self.render_html_code_like(element, context, "[", "]")
    }

    pub(super) fn render_html_select(&mut self, element: ElementRef<'_>) -> Result<()> {
        let options: Vec<_> = element
            .descendants()
            .filter_map(ElementRef::wrap)
            .filter(|option| option.value().name().eq_ignore_ascii_case("option"))
            .collect();
        let multiple = element.attr("multiple").is_some();
        let mut selected: Vec<_> = options
            .iter()
            .copied()
            .filter(|option| option.attr("selected").is_some())
            .collect();
        if selected.is_empty() && !multiple {
            selected.extend(options.first().copied());
        }
        let labels = selected
            .into_iter()
            .map(|option| collapse_control_text(&option.text().collect::<String>()))
            .collect::<Vec<_>>()
            .join(", ");
        self.render_html_control_literal(&format!("[{labels}]"))
    }

    fn render_html_control_literal(&mut self, text: &str) -> Result<()> {
        let stack_len = self.formatting_stack.len();
        self.formatting_stack.push(ThemeElement::Code);
        let result = self.render_html_inline_literal(text);
        self.formatting_stack.truncate(stack_len);
        result
    }
}

fn collapse_control_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
