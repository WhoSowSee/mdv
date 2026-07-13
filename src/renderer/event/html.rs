use super::core::CalloutState;
use super::images::{media_marker, media_marker_leading_separator};
use super::{
    Alignment, CowStr, EventRenderer, HeadingLevel, HtmlBlockBuffer, LinkStyle, Result, TableState,
    ThemeElement, create_style,
};
use crate::utils::{display_width, strip_ansi};
use ego_tree::NodeRef;
use scraper::{ElementRef, Html, Node as HtmlNode};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HtmlAlignment {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug)]
struct HtmlContext {
    alignment: HtmlAlignment,
    preserve_whitespace: bool,
    highlighted: bool,
}

#[derive(Clone, Copy, Debug)]
enum HtmlOrderedListMarkerKind {
    Decimal,
    LowerAlpha,
    UpperAlpha,
    LowerRoman,
    UpperRoman,
}

#[derive(Clone, Copy, Debug)]
enum HtmlListMarkerState {
    Ordered {
        current: i64,
        step: i64,
        kind: HtmlOrderedListMarkerKind,
    },
    Unordered {
        marker: &'static str,
    },
}

impl HtmlListMarkerState {
    fn next_marker(&mut self, item: ElementRef<'_>) -> String {
        match self {
            Self::Ordered {
                current,
                step,
                kind,
            } => {
                if let Some(value) = parse_html_integer_attr(&item, "value") {
                    *current = value;
                }
                let item_kind = html_ordered_list_marker_kind(&item).unwrap_or(*kind);
                let marker = format!("{}. ", format_html_ordered_marker(*current, item_kind));
                *current += *step;
                marker
            }
            Self::Unordered { marker } => (*marker).to_string(),
        }
    }
}

impl Default for HtmlContext {
    fn default() -> Self {
        Self {
            alignment: HtmlAlignment::Left,
            preserve_whitespace: false,
            highlighted: false,
        }
    }
}

impl HtmlContext {
    fn with_alignment(self, alignment: HtmlAlignment) -> Self {
        Self { alignment, ..self }
    }

    fn with_preserve_whitespace(self) -> Self {
        Self {
            preserve_whitespace: true,
            ..self
        }
    }

    fn with_highlighted(self) -> Self {
        Self {
            highlighted: true,
            ..self
        }
    }
}

impl<'a> EventRenderer<'a> {
    pub(super) fn render_html_fragment_buffering_blocks(&mut self, html: &str) -> Result<()> {
        if let Some(buffer) = self.pending_html_block_buffer.as_mut() {
            buffer.content.push_str(html);
            if contains_html_tag(html, buffer.tag, true) {
                self.flush_pending_html_block_buffer()?;
            }
            return Ok(());
        }

        if let Some(tag) = buffering_html_container_tag(html)
            && !contains_html_tag(html, tag, true)
        {
            self.pending_html_block_buffer = Some(HtmlBlockBuffer {
                tag,
                content: html.to_string(),
                captures_markdown_events: false,
            });
            return Ok(());
        }

        if let Some(tag) = buffering_inline_html_container_tag(html)
            && !contains_html_tag(html, tag, true)
        {
            self.pending_html_block_buffer = Some(HtmlBlockBuffer {
                tag,
                content: html.to_string(),
                captures_markdown_events: true,
            });
            return Ok(());
        }

        self.render_html_fragment_as_terminal(html)
    }

    pub(super) fn flush_pending_html_block_buffer(&mut self) -> Result<()> {
        let Some(buffer) = self.pending_html_block_buffer.take() else {
            return Ok(());
        };

        if buffer.content.trim().is_empty() {
            return Ok(());
        }

        self.render_html_fragment_as_terminal(&buffer.content)
    }

    pub(super) fn pending_html_buffer_captures_markdown_events(&self) -> bool {
        self.pending_html_block_buffer
            .as_ref()
            .map(|buffer| buffer.captures_markdown_events)
            .unwrap_or(false)
    }

    pub(super) fn append_pending_html_buffer_text(&mut self, text: &str) -> bool {
        if !self.pending_html_buffer_captures_markdown_events() {
            return false;
        }

        if let Some(buffer) = self.pending_html_block_buffer.as_mut() {
            buffer.content.push_str(&escape_html_text(text));
        }
        true
    }

    pub(super) fn append_pending_html_buffer_soft_break(&mut self) -> bool {
        if !self.pending_html_buffer_captures_markdown_events() {
            return false;
        }

        if let Some(buffer) = self.pending_html_block_buffer.as_mut() {
            buffer.content.push('\n');
        }
        true
    }

    pub(super) fn append_pending_html_buffer_hard_break(&mut self) -> bool {
        if !self.pending_html_buffer_captures_markdown_events() {
            return false;
        }

        if let Some(buffer) = self.pending_html_block_buffer.as_mut() {
            buffer.content.push_str("<br>");
        }
        true
    }

    pub(super) fn render_html_fragment_as_terminal(&mut self, html: &str) -> Result<()> {
        let fragment = Html::parse_fragment(html);
        for node in fragment.tree.root().children() {
            self.render_html_node(node, HtmlContext::default())?;
        }
        self.commit_pending_heading_placeholder_if_content();
        self.flush_html_inline_table_references();
        Ok(())
    }

    fn render_html_node(
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

    fn render_html_element(&mut self, element: ElementRef<'_>, context: HtmlContext) -> Result<()> {
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
            "hr" => self.handle_horizontal_rule(),
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
            "code" | "samp" => self.render_html_code_like(element, child_context, "`", "`"),
            "kbd" => self.render_html_code_like(element, child_context, "[", "]"),
            "mark" => self.render_html_children(element, child_context.with_highlighted()),
            "small" => {
                self.render_html_with_formatting(element, child_context, ThemeElement::TextLight)
            }
            "sub" => self.render_html_code_like(element, child_context, "_", ""),
            "sup" => self.render_html_code_like(element, child_context, "^", ""),
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
            "img" | "video" | "audio" | "source" | "track" | "embed" | "iframe" | "object" => {
                if self.render_html_media(element)? || is_void_html_element(&name) {
                    Ok(())
                } else {
                    self.render_html_children(element, child_context)
                }
            }
            "ul" => self.render_html_list(element, child_context, false),
            "ol" => self.render_html_list(element, child_context, true),
            "li" => self.render_html_list_item(element, child_context, "- "),
            "details" => self.render_html_details(element, child_context),
            "summary" => self.render_html_summary_label(element, child_context),
            _ if is_html_block_element(&name) => self.render_html_block(element, child_context),
            _ if is_void_html_element(&name) => Ok(()),
            _ => self.render_html_children(element, child_context),
        };

        self.formatting_stack.truncate(formatting_stack_len);
        result
    }

    fn render_html_children(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        for child in element.children() {
            self.render_html_node(child, context)?;
        }
        Ok(())
    }

    fn render_html_block(&mut self, element: ElementRef<'_>, context: HtmlContext) -> Result<()> {
        if self.table_state.is_some() {
            self.begin_html_block();
            self.render_html_children(element, context)?;
            self.end_html_block();
            return Ok(());
        }

        self.begin_html_block();
        let content_start = self.output.len();
        self.render_html_children(element, context)?;
        self.align_rendered_html_span(content_start, context.alignment);
        self.end_html_block();
        self.flush_html_inline_table_references();
        Ok(())
    }

    fn render_html_heading(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
        level: HeadingLevel,
    ) -> Result<()> {
        if self.table_state.is_some() {
            self.begin_html_block();
            self.render_html_children(element, context)?;
            self.end_html_block();
            return Ok(());
        }

        self.handle_header_start(level)?;
        let content_start = self.output.len();
        self.render_html_children(element, context)?;
        self.handle_header_end(level)?;
        self.align_rendered_html_span(content_start, context.alignment);
        self.flush_html_inline_table_references();
        Ok(())
    }

    fn render_html_link(&mut self, element: ElementRef<'_>, context: HtmlContext) -> Result<()> {
        let Some(href) = element.attr("href").filter(|href| !href.trim().is_empty()) else {
            return self.render_html_children(element, context);
        };

        self.handle_link_start(CowStr::from(href.to_string()))?;
        self.render_html_children(element, context)?;
        self.handle_link_end()
    }

    fn render_html_with_formatting(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
        theme_element: ThemeElement,
    ) -> Result<()> {
        self.formatting_stack.push(theme_element);
        let result = self.render_html_children(element, context);
        if let Some(index) = self
            .formatting_stack
            .iter()
            .rposition(|current| *current == theme_element)
        {
            self.formatting_stack.remove(index);
        }
        result
    }

    fn render_html_code_like(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
        prefix: &str,
        suffix: &str,
    ) -> Result<()> {
        self.formatting_stack.push(ThemeElement::Code);
        let code_context = context.with_preserve_whitespace();
        let result = self
            .render_html_inline_literal(prefix)
            .and_then(|()| self.render_html_children(element, code_context))
            .and_then(|()| self.render_html_inline_literal(suffix));
        if let Some(index) = self
            .formatting_stack
            .iter()
            .rposition(|current| *current == ThemeElement::Code)
        {
            self.formatting_stack.remove(index);
        }
        result
    }

    fn render_html_abbr(&mut self, element: ElementRef<'_>, context: HtmlContext) -> Result<()> {
        self.render_html_with_formatting(element, context, ThemeElement::TextLight)?;
        if let Some(title) = element
            .attr("title")
            .map(str::trim)
            .filter(|title| !title.is_empty())
        {
            self.render_html_text(&format!(" ({title})"), context)?;
        }
        Ok(())
    }

    fn render_html_inline_literal(&mut self, text: &str) -> Result<()> {
        if text.is_empty() {
            return Ok(());
        }

        self.note_paragraph_content();
        self.process_segment_with_wrapping_and_formatting(text, false, self.table_state.is_some())
    }

    fn render_html_preformatted_block(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        let text = normalize_preformatted_html_text(&element.text().collect::<String>());
        if self.table_state.is_some() {
            self.append_html_table_cell_separator();
            if let Some(ref mut table) = self.table_state {
                table.current_cell.push_str(&text);
            }
            self.append_html_table_cell_separator();
            return Ok(());
        }

        self.begin_html_block();
        let content_start = self.output.len();
        self.render_html_preformatted_text(&text, context)?;
        self.align_rendered_html_span(content_start, context.alignment);
        self.end_html_block();
        Ok(())
    }

    fn render_html_preformatted_text(&mut self, text: &str, context: HtmlContext) -> Result<()> {
        if text.is_empty() {
            return Ok(());
        }

        self.note_paragraph_content();
        for (line_index, line) in text.split('\n').enumerate() {
            if line_index > 0 {
                self.output.push('\n');
            }
            if self.output.is_empty() || self.output.ends_with('\n') {
                self.push_indent_for_line_start();
            }
            let formatted = self.apply_formatting_with_highlight(line, context.highlighted);
            self.output.push_str(&formatted);
        }
        self.commit_pending_heading_placeholder_if_content();
        Ok(())
    }

    fn render_html_blockquote(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        if self.table_state.is_some() {
            return self.render_html_block(element, context);
        }

        self.blockquote_indent_stack
            .push((self.content_indent, self.heading_indent));
        self.blockquote_starts.push(self.output.len());
        self.active_blockquote_smart_indents
            .push(Default::default());
        self.blockquote_level += 1;
        self.current_indent += 2;
        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        self.callout_stack.push(CalloutState::None);

        let result = self.render_html_children(element, context);
        if result.is_ok() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }

        self.callout_stack.pop();
        self.pending_callout_marker = false;
        self.pending_callout_marker_buffer.clear();
        self.pending_callout_label_override = false;
        self.pending_callout_label_buffer.clear();
        self.suppress_next_soft_break = false;
        self.blockquote_level = self.blockquote_level.saturating_sub(1);
        self.current_indent = self.current_indent.saturating_sub(2);
        self.blockquote_starts.pop();
        if let Some((content_indent, heading_indent)) = self.blockquote_indent_stack.pop() {
            self.content_indent = content_indent;
            self.heading_indent = heading_indent;
        }
        self.active_blockquote_smart_indents.pop();
        self.flush_html_inline_table_references();
        result
    }

    fn render_html_definition_list(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        if self.table_state.is_some() {
            return self.render_html_children(element, context);
        }

        self.begin_html_block();
        for child in element.children() {
            self.render_html_node(child, context)?;
        }
        self.end_html_block();
        self.flush_html_inline_table_references();
        Ok(())
    }

    fn render_html_definition_term(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        self.push_indent_for_line_start();
        self.formatting_stack.push(ThemeElement::Strong);
        let result = self.render_html_children(element, context);
        if let Some(index) = self
            .formatting_stack
            .iter()
            .rposition(|current| *current == ThemeElement::Strong)
        {
            self.formatting_stack.remove(index);
        }
        if result.is_ok() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        result
    }

    fn render_html_definition_description(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        self.push_indent_for_line_start();
        let content_start = self.output.len();
        self.note_paragraph_content();
        for child in element.children() {
            if let Some(child_element) = ElementRef::wrap(child)
                && is_definition_description_inline_block(child_element.value().name())
            {
                self.render_html_children(child_element, context)?;
                continue;
            }
            self.render_html_node(child, context)?;
        }
        self.indent_rendered_html_span(content_start, 2);
        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        Ok(())
    }

    fn render_html_figure(&mut self, element: ElementRef<'_>, context: HtmlContext) -> Result<()> {
        if self.table_state.is_some() {
            return self.render_html_children(element, context);
        }

        self.begin_html_block();
        let content_start = self.output.len();
        for child in element.children() {
            if let Some(child_element) = ElementRef::wrap(child)
                && child_element
                    .value()
                    .name()
                    .eq_ignore_ascii_case("figcaption")
            {
                continue;
            }
            self.render_html_node(child, context)?;
        }
        for caption in element
            .child_elements()
            .filter(|child| child.value().name().eq_ignore_ascii_case("figcaption"))
        {
            self.render_html_figcaption(caption, context)?;
        }
        self.align_rendered_html_span(content_start, context.alignment);
        self.end_html_block();
        self.flush_html_inline_table_references();
        Ok(())
    }

    fn render_html_figcaption(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        self.push_indent_for_line_start();
        self.formatting_stack.push(ThemeElement::TextLight);
        self.formatting_stack.push(ThemeElement::Emphasis);
        let result = self.render_html_children(element, context);
        if let Some(index) = self
            .formatting_stack
            .iter()
            .rposition(|current| *current == ThemeElement::Emphasis)
        {
            self.formatting_stack.remove(index);
        }
        if let Some(index) = self
            .formatting_stack
            .iter()
            .rposition(|current| *current == ThemeElement::TextLight)
        {
            self.formatting_stack.remove(index);
        }
        if result.is_ok() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        result
    }

    fn render_html_list(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
        ordered: bool,
    ) -> Result<()> {
        let mut marker_state = html_list_marker_state(&element, ordered);
        if self.table_state.is_some() {
            for child in element.children() {
                if let Some(child_element) = ElementRef::wrap(child)
                    && child_element.value().name().eq_ignore_ascii_case("li")
                {
                    let marker = marker_state.next_marker(child_element);
                    self.render_html_list_item(child_element, context, &marker)?;
                    continue;
                }
                self.render_html_node(child, context)?;
            }
            return Ok(());
        }

        self.begin_html_block();
        for child in element.children() {
            if let Some(child_element) = ElementRef::wrap(child)
                && child_element.value().name().eq_ignore_ascii_case("li")
            {
                let marker = marker_state.next_marker(child_element);
                self.render_html_list_item(child_element, context, &marker)?;
                continue;
            }
            self.render_html_node(child, context)?;
        }
        self.end_html_block();
        self.flush_html_inline_table_references();
        Ok(())
    }

    fn render_html_list_item(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
        marker: &str,
    ) -> Result<()> {
        if self.table_state.is_some() {
            self.append_html_table_cell_separator();
            if let Some(ref mut table) = self.table_state {
                table.current_cell.push_str(marker);
            }
            self.render_html_children(element, context)?;
            self.append_html_table_cell_separator();
            return Ok(());
        }

        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        self.push_indent_for_line_start();
        self.output.push_str(marker);
        self.note_paragraph_content();
        self.render_html_children(element, context)?;
        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        Ok(())
    }

    fn render_html_details(&mut self, element: ElementRef<'_>, context: HtmlContext) -> Result<()> {
        if self.table_state.is_some() {
            return self.render_html_children(element, context);
        }

        self.begin_html_block();
        let content_start = self.output.len();
        let mut rendered_summary = false;
        for child in element.children() {
            if let Some(child_element) = ElementRef::wrap(child)
                && child_element.value().name().eq_ignore_ascii_case("summary")
                && !rendered_summary
            {
                self.render_html_summary_label(child_element, context)?;
                rendered_summary = true;
                continue;
            }
            self.render_html_node(child, context)?;
        }
        self.align_rendered_html_span(content_start, context.alignment);
        self.end_html_block();
        self.flush_html_inline_table_references();
        Ok(())
    }

    fn render_html_summary_label(
        &mut self,
        element: ElementRef<'_>,
        context: HtmlContext,
    ) -> Result<()> {
        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }

        self.push_indent_for_line_start();
        self.formatting_stack.push(ThemeElement::Strong);
        let result = self.render_html_children(element, context);
        if let Some(index) = self
            .formatting_stack
            .iter()
            .rposition(|current| *current == ThemeElement::Strong)
        {
            self.formatting_stack.remove(index);
        }
        if result.is_ok() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        result
    }

    fn render_html_table(&mut self, element: ElementRef<'_>, context: HtmlContext) -> Result<()> {
        if self.table_state.is_some() {
            return self.render_html_children(element, context);
        }

        for caption in element
            .child_elements()
            .filter(|child| child.value().name().eq_ignore_ascii_case("caption"))
        {
            self.render_html_block(caption, context)?;
        }

        if matches!(self.config.link_style, LinkStyle::InlineTable) {
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
            inline_references: Vec::new(),
            inline_url_segments: Vec::new(),
        });

        self.render_html_table_section(element, context, false)?;

        let Some(mut table) = self.table_state.take() else {
            return Ok(());
        };
        normalize_html_table(&mut table);
        let table_indent = self.render_table(table)?;

        if matches!(self.config.link_style, LinkStyle::InlineTable)
            && !self.paragraph_links.is_empty()
        {
            self.add_paragraph_link_references_for_table(table_indent);
        }

        Ok(())
    }

    fn render_html_table_section(
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

    fn render_html_table_row(
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

    fn render_html_table_cell(
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

    fn render_html_media(&mut self, element: ElementRef<'_>) -> Result<bool> {
        let Some(source) = html_media_source(&element) else {
            return Ok(false);
        };

        let label = html_media_label(&element, &source);
        if let Some(ref mut table) = self.table_state {
            let marker = media_marker(&source);
            let separator = media_marker_leading_separator(&table.current_cell);
            table.current_cell.push_str(separator);
            table.current_cell.push_str(marker);
            table.current_cell.push_str(&label);
            self.commit_pending_heading_placeholder_if_content();
            return Ok(true);
        }

        let marker = media_marker(&source);
        self.note_paragraph_content();
        self.prepare_html_media_line(marker, &label);

        let style = create_style(self.theme, ThemeElement::Link);
        let styled_marker = style.apply(marker, self.config.no_colors);
        let separator = media_marker_leading_separator(&self.output);
        self.output.push_str(separator);
        self.output.push_str(&styled_marker);
        if !label.is_empty() {
            self.process_segment_with_wrapping_and_formatting(&label, false, false)?;
        }
        self.commit_pending_heading_placeholder_if_content();
        Ok(true)
    }

    fn prepare_html_media_line(&mut self, marker: &str, label: &str) {
        let line_start_idx = self.output.rfind('\n').map(|idx| idx + 1).unwrap_or(0);
        let current_line = &self.output[line_start_idx..];
        if current_line.trim().is_empty() {
            self.output.truncate(line_start_idx);
            self.push_indent_for_line_start();
            return;
        }

        if !self.config.is_text_wrapping_enabled() {
            return;
        }

        let current_line_clean = strip_ansi(current_line);
        let current_width = display_width(&current_line_clean);
        let media_width = display_width(media_marker_leading_separator(&self.output))
            + display_width(marker)
            + display_width(label);
        let would_exceed = current_width + media_width > self.effective_text_width();
        let has_visible_text = current_line_clean
            .chars()
            .any(|ch| !ch.is_whitespace() && ch != '│' && ch != '┃');

        if would_exceed && current_width > 0 && has_visible_text {
            self.push_newline_with_context();
        }
    }

    fn render_html_text(&mut self, text: &str, context: HtmlContext) -> Result<()> {
        let text = if context.preserve_whitespace {
            text.replace("\r\n", "\n").replace('\r', "\n")
        } else {
            self.collapse_html_text(text)
        };

        if text.is_empty() {
            return Ok(());
        }

        if context.highlighted {
            self.note_paragraph_content();
            return self.process_segment_with_wrapping_and_formatting(
                &text,
                true,
                self.table_state.is_some(),
            );
        }

        self.handle_text(CowStr::from(text))
    }

    fn collapse_html_text(&self, text: &str) -> String {
        let starts_with_whitespace = text
            .chars()
            .next()
            .map(char::is_whitespace)
            .unwrap_or(false);
        let ends_with_whitespace = text
            .chars()
            .next_back()
            .map(char::is_whitespace)
            .unwrap_or(false);

        let mut collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
        if collapsed.is_empty() {
            if starts_with_whitespace && self.needs_html_separator_before_text() {
                return " ".to_string();
            }
            return String::new();
        }

        if starts_with_whitespace && self.needs_html_separator_before_text() {
            collapsed.insert(0, ' ');
        }
        if ends_with_whitespace {
            collapsed.push(' ');
        }

        collapsed
    }

    fn needs_html_separator_before_text(&self) -> bool {
        let line = self
            .output
            .rsplit_once('\n')
            .map(|(_, line)| line)
            .unwrap_or(&self.output);
        let clean = strip_ansi(line);
        clean
            .chars()
            .next_back()
            .map(|ch| !ch.is_whitespace() && !matches!(ch, '(' | '[' | '{' | '/' | ' '))
            .unwrap_or(false)
    }

    fn render_html_line_break(&mut self) {
        if let Some(ref mut table) = self.table_state {
            if !table.current_cell.ends_with('\n') {
                table.current_cell.push('\n');
            }
            return;
        }

        if self.output.is_empty() {
            return;
        }
        self.push_newline_with_context();
    }

    fn begin_html_block(&mut self) {
        if self.table_state.is_some() {
            self.append_html_table_cell_separator();
            return;
        }

        if self.output.is_empty() {
            return;
        }
        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }
    }

    fn end_html_block(&mut self) {
        if self.table_state.is_some() {
            self.append_html_table_cell_separator();
            return;
        }

        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
    }

    fn append_html_table_cell_separator(&mut self) {
        if let Some(ref mut table) = self.table_state {
            let needs_space = table
                .current_cell
                .chars()
                .next_back()
                .map(|ch| !ch.is_whitespace())
                .unwrap_or(false);
            if needs_space {
                table.current_cell.push(' ');
            }
        }
    }

    fn push_html_inline_style_elements(&mut self, element: &ElementRef<'_>) {
        for theme_element in html_inline_style_elements(element) {
            if !self.formatting_stack.contains(&theme_element) {
                self.formatting_stack.push(theme_element);
            }
        }
    }

    fn flush_html_inline_table_references(&mut self) {
        if matches!(self.config.link_style, LinkStyle::InlineTable)
            && self.table_state.is_none()
            && self.current_paragraph_start.is_none()
            && !self.paragraph_links.is_empty()
        {
            self.add_paragraph_link_references();
            self.ensure_contextual_blank_line();
        }
    }

    fn align_rendered_html_span(&mut self, start: usize, alignment: HtmlAlignment) {
        if matches!(alignment, HtmlAlignment::Left) {
            return;
        }
        if start >= self.output.len() {
            return;
        }

        let prefix = self.current_line_prefix();
        let span = self.output[start..].to_string();
        let mut aligned = String::new();
        let mut lines = span.split('\n').peekable();

        while let Some(line) = lines.next() {
            let has_more = lines.peek().is_some();
            aligned.push_str(&self.align_html_line(line, &prefix, alignment));
            if has_more {
                aligned.push('\n');
            }
        }

        self.output.replace_range(start.., &aligned);
    }

    fn indent_rendered_html_span(&mut self, start: usize, indent: usize) {
        if start >= self.output.len() || indent == 0 {
            return;
        }

        let prefix = self.current_line_prefix();
        let span = self.output[start..].to_string();
        let mut indented = String::new();
        let mut lines = span.split('\n').peekable();

        while let Some(line) = lines.next() {
            let has_more = lines.peek().is_some();
            if strip_ansi(line).trim().is_empty() {
                indented.push_str(line);
            } else if !prefix.is_empty() && line.starts_with(&prefix) {
                indented.push_str(&prefix);
                indented.push_str(&" ".repeat(indent));
                indented.push_str(&line[prefix.len()..]);
            } else {
                indented.push_str(&" ".repeat(indent));
                indented.push_str(line);
            }

            if has_more {
                indented.push('\n');
            }
        }

        self.output.replace_range(start.., &indented);
    }

    fn align_html_line(&self, line: &str, prefix: &str, alignment: HtmlAlignment) -> String {
        if strip_ansi(line).trim().is_empty() {
            return line.to_string();
        }

        let (line_prefix, content) = if !prefix.is_empty() && line.starts_with(prefix) {
            (prefix, &line[prefix.len()..])
        } else {
            ("", line)
        };
        let content = content.trim();
        let content_width = display_width(&strip_ansi(content));
        let prefix_width = display_width(&strip_ansi(line_prefix));
        let available_width = self.effective_text_width().saturating_sub(prefix_width);
        let padding = match alignment {
            HtmlAlignment::Left => 0,
            HtmlAlignment::Center => available_width.saturating_sub(content_width) / 2,
            HtmlAlignment::Right => available_width.saturating_sub(content_width),
        };

        format!("{line_prefix}{}{content}", " ".repeat(padding))
    }
}

fn html_alignment(element: &ElementRef<'_>) -> Option<HtmlAlignment> {
    if element.value().name().eq_ignore_ascii_case("center") {
        return Some(HtmlAlignment::Center);
    }

    if let Some(alignment) = element.attr("align").and_then(parse_html_alignment) {
        return Some(alignment);
    }

    element.attr("style").and_then(html_text_align_from_style)
}

fn html_inline_style_elements(element: &ElementRef<'_>) -> Vec<ThemeElement> {
    let Some(style) = element.attr("style") else {
        return Vec::new();
    };

    let mut elements = Vec::new();
    for declaration in style.split(';') {
        let Some((property, value)) = declaration.split_once(':') else {
            continue;
        };
        let property = property.trim();
        let value = value.trim();

        if property.eq_ignore_ascii_case("font-weight") && is_bold_font_weight(value) {
            push_unique_theme_element(&mut elements, ThemeElement::Strong);
        } else if property.eq_ignore_ascii_case("font-style")
            && matches_ignore_ascii_case_any(value, &["italic", "oblique"])
        {
            push_unique_theme_element(&mut elements, ThemeElement::Emphasis);
        } else if matches_ignore_ascii_case_any(
            property,
            &["text-decoration", "text-decoration-line"],
        ) {
            for token in value.split_whitespace() {
                if token.eq_ignore_ascii_case("line-through") {
                    push_unique_theme_element(&mut elements, ThemeElement::Strikethrough);
                } else if token.eq_ignore_ascii_case("underline") {
                    push_unique_theme_element(&mut elements, ThemeElement::Underline);
                }
            }
        }
    }

    elements
}

fn push_unique_theme_element(elements: &mut Vec<ThemeElement>, element: ThemeElement) {
    if !elements.contains(&element) {
        elements.push(element);
    }
}

fn is_bold_font_weight(value: &str) -> bool {
    if matches_ignore_ascii_case_any(value, &["bold", "bolder"]) {
        return true;
    }

    value
        .parse::<u16>()
        .map(|weight| weight >= 600)
        .unwrap_or(false)
}

fn matches_ignore_ascii_case_any(value: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| value.eq_ignore_ascii_case(candidate))
}

fn html_list_marker_state(element: &ElementRef<'_>, ordered: bool) -> HtmlListMarkerState {
    if ordered {
        let reversed = element.attr("reversed").is_some();
        let default_start = if reversed {
            html_direct_list_item_count(element).max(1) as i64
        } else {
            1
        };
        return HtmlListMarkerState::Ordered {
            current: parse_html_integer_attr(element, "start").unwrap_or(default_start),
            step: if reversed { -1 } else { 1 },
            kind: html_ordered_list_marker_kind(element)
                .unwrap_or(HtmlOrderedListMarkerKind::Decimal),
        };
    }

    HtmlListMarkerState::Unordered {
        marker: html_unordered_list_marker(element),
    }
}

fn html_direct_list_item_count(element: &ElementRef<'_>) -> usize {
    element
        .child_elements()
        .filter(|child| child.value().name().eq_ignore_ascii_case("li"))
        .count()
}

fn parse_html_integer_attr(element: &ElementRef<'_>, attr: &str) -> Option<i64> {
    element.attr(attr)?.trim().parse::<i64>().ok()
}

fn html_ordered_list_marker_kind(element: &ElementRef<'_>) -> Option<HtmlOrderedListMarkerKind> {
    match element.attr("type")?.trim() {
        "1" => Some(HtmlOrderedListMarkerKind::Decimal),
        "a" => Some(HtmlOrderedListMarkerKind::LowerAlpha),
        "A" => Some(HtmlOrderedListMarkerKind::UpperAlpha),
        "i" => Some(HtmlOrderedListMarkerKind::LowerRoman),
        "I" => Some(HtmlOrderedListMarkerKind::UpperRoman),
        _ => None,
    }
}

fn html_unordered_list_marker(element: &ElementRef<'_>) -> &'static str {
    match element.attr("type").map(str::trim) {
        Some(value) if value.eq_ignore_ascii_case("disc") => "• ",
        Some(value) if value.eq_ignore_ascii_case("circle") => "◦ ",
        Some(value) if value.eq_ignore_ascii_case("square") => "▪ ",
        _ => "- ",
    }
}

fn format_html_ordered_marker(value: i64, kind: HtmlOrderedListMarkerKind) -> String {
    match kind {
        HtmlOrderedListMarkerKind::Decimal => value.to_string(),
        HtmlOrderedListMarkerKind::LowerAlpha => format_alpha_marker(value, false),
        HtmlOrderedListMarkerKind::UpperAlpha => format_alpha_marker(value, true),
        HtmlOrderedListMarkerKind::LowerRoman => format_roman_marker(value, false),
        HtmlOrderedListMarkerKind::UpperRoman => format_roman_marker(value, true),
    }
}

fn format_alpha_marker(value: i64, uppercase: bool) -> String {
    if value <= 0 {
        return value.to_string();
    }

    let mut remaining = value;
    let mut marker = Vec::new();
    while remaining > 0 {
        remaining -= 1;
        let base = if uppercase { b'A' } else { b'a' };
        marker.push((base + (remaining % 26) as u8) as char);
        remaining /= 26;
    }

    marker.iter().rev().collect()
}

fn format_roman_marker(value: i64, uppercase: bool) -> String {
    if !(1..=3999).contains(&value) {
        return value.to_string();
    }

    let mut remaining = value;
    let mut marker = String::new();
    for (amount, symbol) in [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ] {
        while remaining >= amount {
            marker.push_str(symbol);
            remaining -= amount;
        }
    }

    if uppercase {
        marker
    } else {
        marker.to_ascii_lowercase()
    }
}

fn parse_html_alignment(value: &str) -> Option<HtmlAlignment> {
    match value.trim().to_ascii_lowercase().as_str() {
        "center" | "middle" => Some(HtmlAlignment::Center),
        "right" => Some(HtmlAlignment::Right),
        "left" => Some(HtmlAlignment::Left),
        _ => None,
    }
}

fn table_alignment_from_html(alignment: HtmlAlignment) -> Alignment {
    match alignment {
        HtmlAlignment::Left => Alignment::Left,
        HtmlAlignment::Center => Alignment::Center,
        HtmlAlignment::Right => Alignment::Right,
    }
}

fn html_text_align_from_style(style: &str) -> Option<HtmlAlignment> {
    for declaration in style.split(';') {
        let Some((property, value)) = declaration.split_once(':') else {
            continue;
        };
        if property.trim().eq_ignore_ascii_case("text-align")
            && let Some(alignment) = parse_html_alignment(value)
        {
            return Some(alignment);
        }
    }

    None
}

fn normalize_html_table(table: &mut TableState) {
    if table.headers.is_empty() && !table.rows.is_empty() {
        table.headers = table.rows.remove(0);
    }

    let column_count = table
        .headers
        .len()
        .max(table.rows.iter().map(Vec::len).max().unwrap_or(0));
    if column_count == 0 {
        return;
    }

    if table.headers.len() < column_count {
        table.headers.extend(std::iter::repeat_n(
            String::new(),
            column_count - table.headers.len(),
        ));
    }
    if table.alignments.len() < column_count {
        table.alignments.extend(std::iter::repeat_n(
            Alignment::None,
            column_count - table.alignments.len(),
        ));
    }
    for row in &mut table.rows {
        if row.len() < column_count {
            row.extend(std::iter::repeat_n(String::new(), column_count - row.len()));
        }
    }
}

fn html_media_source(element: &ElementRef<'_>) -> Option<String> {
    for attr in ["src", "data", "href"] {
        if let Some(value) = element.attr(attr).map(str::trim)
            && !value.is_empty()
        {
            return Some(value.to_string());
        }
    }

    element
        .attr("srcset")
        .and_then(first_srcset_candidate)
        .map(str::to_string)
}

fn first_srcset_candidate(srcset: &str) -> Option<&str> {
    srcset
        .split(',')
        .filter_map(|candidate| candidate.split_whitespace().next())
        .find(|candidate| !candidate.is_empty())
}

fn html_media_label(element: &ElementRef<'_>, source: &str) -> String {
    for attr in ["alt", "title", "aria-label"] {
        if let Some(value) = element.attr(attr).map(str::trim)
            && !value.is_empty()
        {
            return value.to_string();
        }
    }

    media_filename(source).unwrap_or(source).to_string()
}

fn media_filename(source: &str) -> Option<&str> {
    let path = source.split(['?', '#']).next().unwrap_or(source);
    let filename = path.rsplit(['/', '\\']).next().unwrap_or(path).trim();
    if filename.is_empty() {
        None
    } else {
        Some(filename)
    }
}

fn buffering_html_container_tag(html: &str) -> Option<&'static str> {
    BUFFERED_HTML_CONTAINER_TAGS
        .iter()
        .copied()
        .find(|tag| contains_html_tag(html, tag, false))
}

fn buffering_inline_html_container_tag(html: &str) -> Option<&'static str> {
    ["a"]
        .into_iter()
        .find(|tag| contains_html_tag(html, tag, false))
}

fn contains_html_tag(html: &str, tag: &str, closing: bool) -> bool {
    let lower = html.to_ascii_lowercase();
    let needle = if closing {
        format!("</{tag}")
    } else {
        format!("<{tag}")
    };
    let mut offset = 0;

    while let Some(index) = lower[offset..].find(&needle) {
        let after = offset + index + needle.len();
        let has_tag_boundary = lower[after..]
            .chars()
            .next()
            .map(|ch| ch == '>' || ch == '/' || ch.is_ascii_whitespace())
            .unwrap_or(false);

        if has_tag_boundary {
            return true;
        }

        offset = after;
    }

    false
}

fn is_html_block_element(name: &str) -> bool {
    matches!(
        name,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "dd"
            | "details"
            | "dialog"
            | "div"
            | "dl"
            | "dt"
            | "fieldset"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "header"
            | "main"
            | "nav"
            | "p"
            | "section"
            | "summary"
            | "center"
    )
}

fn is_definition_description_inline_block(name: &str) -> bool {
    matches!(name, "p" | "div" | "section" | "article" | "span")
}

const BUFFERED_HTML_CONTAINER_TAGS: &[&str] = &[
    "table",
    "p",
    "div",
    "center",
    "section",
    "figure",
    "header",
    "footer",
    "main",
    "article",
    "aside",
    "nav",
    "details",
    "blockquote",
    "dl",
    "ol",
    "pre",
    "textarea",
    "ul",
];

fn normalize_preformatted_html_text(text: &str) -> String {
    let mut normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    if normalized.starts_with('\n') {
        normalized.remove(0);
    }
    if normalized.ends_with('\n') {
        normalized.pop();
    }
    normalized
}

fn escape_html_text(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn is_void_html_element(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}
