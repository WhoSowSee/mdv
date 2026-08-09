use super::{EventRenderer, ThemeElement, create_style};

struct ActiveDefinitionDescription {
    output_start: usize,
    content_start: usize,
    previous_content_indent: Option<usize>,
}

#[derive(Default)]
pub(super) struct DefinitionListState {
    has_term: bool,
    has_description: bool,
    active_description: Option<ActiveDefinitionDescription>,
}

impl EventRenderer<'_> {
    pub(super) fn in_definition_description(&self) -> bool {
        self.definition_list_stack
            .iter()
            .any(|state| state.active_description.is_some())
    }

    pub(super) fn handle_definition_list_start(&mut self) {
        self.ensure_contextual_blank_line();
        self.definition_list_stack
            .push(DefinitionListState::default());
    }

    pub(super) fn handle_definition_title_start(&mut self) {
        let has_previous_term = self
            .definition_list_stack
            .last()
            .is_some_and(|state| state.has_term);

        if has_previous_term {
            self.ensure_contextual_blank_line();
        } else if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }

        self.push_indent_for_line_start();

        if let Some(state) = self.definition_list_stack.last_mut() {
            state.has_term = true;
            state.has_description = false;
        }
    }

    pub(super) fn handle_definition_description_start(&mut self) {
        let output_start = self.output.len();
        let has_previous_description = self
            .definition_list_stack
            .last()
            .is_some_and(|state| state.has_description);

        if has_previous_description {
            self.ensure_contextual_blank_line();
        } else if !self.output.ends_with('\n') {
            self.output.push('\n');
        }

        let previous_content_indent = if let Some(style) = self.config.pretty_definition {
            self.push_indent_for_line_start();
            let marker = create_style(self.theme, ThemeElement::ListMarker)
                .apply(style.marker(), self.config.no_colors);
            self.output.push_str(&marker);
            None
        } else {
            let previous_indent = self.content_indent;
            self.content_indent = self.content_indent.saturating_add(2);
            self.push_indent_for_line_start();
            Some(previous_indent)
        };
        let content_start = self.output.len();

        if let Some(state) = self.definition_list_stack.last_mut() {
            state.active_description = Some(ActiveDefinitionDescription {
                output_start,
                content_start,
                previous_content_indent,
            });
        }
    }

    pub(super) fn handle_definition_description_end(&mut self) {
        let Some(description) = self
            .definition_list_stack
            .last_mut()
            .and_then(|state| state.active_description.take())
        else {
            return;
        };

        let content_start = description.content_start.min(self.output.len());
        let has_visible_content = self.output[content_start..]
            .chars()
            .any(|ch| !ch.is_whitespace());

        if !has_visible_content && !self.config.show_empty_elements {
            self.output
                .truncate(description.output_start.min(self.output.len()));
        } else {
            if !self.output.ends_with('\n') {
                self.output.push('\n');
            }
            if let Some(state) = self.definition_list_stack.last_mut() {
                state.has_description = true;
            }
        }

        if let Some(previous_content_indent) = description.previous_content_indent {
            self.content_indent = previous_content_indent;
        }
    }

    pub(super) fn handle_definition_list_end(&mut self) {
        let had_terms = self
            .definition_list_stack
            .pop()
            .is_some_and(|state| state.has_term);
        if had_terms {
            self.ensure_contextual_blank_line();
        }
    }
}
