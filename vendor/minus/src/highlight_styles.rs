use crate::PromptColor;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HighlightColors {
    pub foreground: Option<PromptColor>,
    pub background: Option<PromptColor>,
}

impl HighlightColors {
    pub(crate) fn sequence(
        self,
        foreground: Option<PromptColor>,
        background: Option<PromptColor>,
    ) -> String {
        let mut output = String::new();
        if let Some(color) = self.foreground.or(foreground) {
            crate::prompt::write_prompt_color(&mut output, color, false);
        }
        if let Some(color) = self.background.or(background) {
            let mut background_codes = String::new();
            crate::prompt::write_prompt_color(&mut background_codes, color, true);
            if output.is_empty() {
                output = background_codes;
            } else {
                output.pop();
                output.push(';');
                output.push_str(&background_codes[2..]);
            }
        }
        output
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HighlightStyles {
    pub selection: HighlightColors,
    pub search: HighlightColors,
    pub search_current: HighlightColors,
}

impl HighlightStyles {
    pub(crate) const fn search_colors(self, current: bool) -> HighlightColors {
        if current {
            self.search_current
        } else {
            self.search
        }
    }
}
