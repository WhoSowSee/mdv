use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn render_html_node(
        &mut self,
        node: NodeRef<'_, HtmlNode>,
        context: HtmlContext,
    ) -> Result<()> {
        match node.value() {
            HtmlNode::Text(text) => self.render_html_text(text.as_ref(), context)?,
            HtmlNode::Element(_) => {
                if let Some(element) = ElementRef::wrap(node) {
                    self.render_html_element(element, context)?;
                }
            }
            HtmlNode::Document | HtmlNode::Fragment => {
                for child in node.children() {
                    self.render_html_node(child, context)?;
                }
            }
            HtmlNode::Comment(_) | HtmlNode::Doctype(_) | HtmlNode::ProcessingInstruction(_) => {}
        }

        Ok(())
    }

    pub(super) fn render_html_element(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        let name = element.value().name().to_ascii_lowercase();
        if matches!(
            name.as_str(),
            "script" | "style" | "template" | "noscript" | "title"
        ) {
            return Ok(());
        }

        let alignment = html_alignment(&element).unwrap_or(context.alignment);
        let child_context = context.with_alignment(alignment);
        let formatting_stack_len = self.formatting_stack.len();
        self.push_html_inline_style_elements(&element);

        let result = match name.as_str() {
            "html" | "body" => self.render_html_children(element, child_context),
            "head" => Ok(()),
            "br" | "wbr" => {
                self.render_html_line_break();
                Ok(())
            }
            "hr" => {
                if self.table_state.is_some() {
                    self.render_html_table_horizontal_rule(child_context);
                    Ok(())
                } else {
                    self.handle_horizontal_rule()
                }
            }
            "a" => self.render_html_link(element, child_context),
            "strong" | "b" => {
                self.render_html_with_formatting(element, child_context, ThemeElement::Strong)
            }
            "em" | "i" | "cite" => {
                self.render_html_with_formatting(element, child_context, ThemeElement::Emphasis)
            }
            "s" | "strike" | "del" => self.render_html_with_formatting(
                element,
                child_context,
                ThemeElement::Strikethrough,
            ),
            "code" | "samp" => {
                let marker = if self
                    .theme
                    .inline_style
                    .get(crate::inline_style::InlineStyleKind::Code)
                    .backticks
                {
                    "`"
                } else {
                    ""
                };
                self.render_html_code_like(element, child_context, marker, marker)
            }
            "kbd" => self.render_html_code_like(element, child_context, "[", "]"),
            "mark" => self.render_html_children(element, child_context.with_highlighted()),
            "small" => {
                self.render_html_with_formatting(element, child_context, ThemeElement::TextLight)
            }
            "sub" => self.render_html_children(element, child_context.with_script(ScriptKind::Sub)),
            "sup" => self.render_html_children(element, child_context.with_script(ScriptKind::Sup)),
            "abbr" => self.render_html_abbr(element, child_context),
            "pre" | "textarea" => self.render_html_preformatted_block(element, child_context),
            "h1" => self.render_html_heading(element, child_context, HeadingLevel::H1),
            "h2" => self.render_html_heading(element, child_context, HeadingLevel::H2),
            "h3" => self.render_html_heading(element, child_context, HeadingLevel::H3),
            "h4" => self.render_html_heading(element, child_context, HeadingLevel::H4),
            "h5" => self.render_html_heading(element, child_context, HeadingLevel::H5),
            "h6" => self.render_html_heading(element, child_context, HeadingLevel::H6),
            "table" => self.render_html_table(element, child_context),
            "figure" => self.render_html_figure(element, child_context),
            "figcaption" => self.render_html_figcaption(element, child_context),
            "blockquote" => self.render_html_blockquote(element, child_context),
            "dl" => self.render_html_definition_list(element, child_context),
            "dt" => self.render_html_definition_term(element, child_context),
            "dd" => self.render_html_definition_description(element, child_context),
            "thead" | "tbody" | "tfoot" | "tr" | "th" | "td" | "caption" | "colgroup" => {
                self.render_html_children(element, child_context)
            }
            "input" => self.render_html_input(element),
            "button" => self.render_html_button(element, child_context),
            "select" => self.render_html_select(element),
            "img" | "video" | "audio" | "source" | "track" | "embed" | "iframe" | "object" => {
                if self.render_html_media(element)? || is_void_html_element(&name) {
                    Ok(())
                } else {
                    self.render_html_children(element, child_context)
                }
            }
            "ul" => self.render_html_list(element, child_context, false),
            "ol" => self.render_html_list(element, child_context, true),
            "li" => self.render_html_list_item(
                element,
                child_context,
                "- ",
                Some(child_context.list_depth.saturating_add(1)),
            ),
            "details" => self.render_html_details(element, child_context),
            "summary" => self.render_html_summary_label(element, child_context),
            _ if is_html_block_element(&name) => self.render_html_block(element, child_context),
            _ if is_void_html_element(&name) => Ok(()),
            _ => self.render_html_children(element, child_context),
        };

        self.formatting_stack.truncate(formatting_stack_len);
        result
    }

    pub(super) fn render_html_children(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        for child in element.children() {
            self.render_html_node(child, context)?;
        }
        Ok(())
    }
}
