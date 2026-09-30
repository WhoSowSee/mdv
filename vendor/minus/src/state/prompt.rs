use super::{PagerState, PromptContext, PromptError};

impl PagerState {
    pub(crate) fn format_prompt(&mut self) -> Result<(), PromptError> {
        if let Some(renderer) = &self.prompt_renderer {
            let prompt = renderer(&PromptContext::new(self))?;
            self.displayed_prompt = if self.output_styling {
                prompt.render(self.cols)
            } else {
                prompt.render_plain(self.cols)
            };
        } else {
            self.format_default_prompt();
        }
        self.displayed_prompt = self
            .color_depth
            .adapt(std::mem::take(&mut self.displayed_prompt))
            .into_owned();

        self.displayed_prompt_panel = self
            .prompt_panel
            .iter()
            .map(|line| {
                if self.output_styling {
                    self.color_depth.adapt(line.render(self.cols)).into_owned()
                } else {
                    line.render_plain(self.cols)
                }
            })
            .collect();
        Ok(())
    }

    pub(super) fn format_default_prompt(&mut self) {
        const PROMPT_SPEC: &str = "\x1b[2;40;37m";
        const SEARCH_SPEC: &str = "\x1b[30;44m";
        const INPUT_SPEC: &str = "\x1b[30;43m";
        const MSG_SPEC: &str = "\x1b[30;1;41m";
        const RESET: &str = "\x1b[0m";
        const FOLLOW_MODE_SPEC: &str = "\x1b[1m";

        let mut format_string = String::with_capacity(self.cols + (SEARCH_SPEC.len() * 5) + 4);

        #[cfg(feature = "search")]
        let mut search_str = String::new();
        #[cfg(feature = "search")]
        if !self.search_state.search_matches.is_empty() {
            search_str.push(' ');
            search_str.push_str(&(self.search_state.search_mark + 1).to_string());
            search_str.push('/');
            search_str.push_str(&self.search_state.search_matches.len().to_string());
            search_str.push(' ');
        }

        let mut prefix_str = String::new();
        if !self.prefix_num.is_empty() {
            prefix_str.push(' ');
            prefix_str.push_str(&self.prefix_num);
            prefix_str.push(' ');
        }

        let prompt_str = self.message.as_ref().unwrap_or(&self.prompt);

        #[cfg(feature = "search")]
        let search_len = search_str.len();
        #[cfg(not(feature = "search"))]
        let search_len = 0;

        let follow_mode_str: &str = if self.follow_output { "[F]" } else { "" };

        // Prompt width counts Unicode characters rather than UTF-8 bytes.
        let prefix_len = prefix_str.len();
        let extra_space = self.cols.saturating_sub(
            search_len + prefix_len + follow_mode_str.len() + prompt_str.chars().count(),
        );

        let byte_idx = prompt_str
            .char_indices()
            .nth(search_len + prefix_len + follow_mode_str.len());

        let dsp_prompt: &str = if extra_space == 0
            && let Some((idx, _)) = byte_idx
        {
            &prompt_str[..idx]
        } else {
            prompt_str
        };

        if self.output_styling && self.message.is_some() {
            format_string.push_str(MSG_SPEC);
        } else if self.output_styling {
            format_string.push_str(PROMPT_SPEC);
        }
        format_string.push_str(dsp_prompt);
        format_string.push_str(&" ".repeat(extra_space));

        if prefix_len > 0 {
            if self.output_styling {
                format_string.push_str(INPUT_SPEC);
            }
            format_string.push_str(&prefix_str);
        }

        #[cfg(feature = "search")]
        if search_len > 0 {
            if self.output_styling {
                format_string.push_str(SEARCH_SPEC);
            }
            format_string.push_str(&search_str);
        }

        if !follow_mode_str.is_empty() {
            if self.output_styling {
                format_string.push_str(FOLLOW_MODE_SPEC);
            }
            format_string.push_str(follow_mode_str);
        }

        if self.output_styling {
            format_string.push_str(RESET);
        }

        self.displayed_prompt = format_string;
    }
}
