use super::*;

impl Config {
    pub fn text_wrap_mode(&self) -> crate::utils::WrapMode {
        match self.wrap {
            TextWrapMode::Char => crate::utils::WrapMode::Character,
            TextWrapMode::Word => crate::utils::WrapMode::Word,
            TextWrapMode::None => crate::utils::WrapMode::None,
        }
    }

    pub fn is_text_wrapping_enabled(&self) -> bool {
        !matches!(self.wrap, TextWrapMode::None)
    }

    pub fn get_terminal_width(&self) -> usize {
        if self.cols_from_cli
            && let Some(cols) = self.cols
        {
            return cols;
        }

        if let Ok((width, _)) = crossterm::terminal::size() {
            let width = width as usize;
            if width >= 20 {
                return width;
            }
        }

        if let Some(cols) = self.cols {
            return cols;
        }

        80 // Default fallback
    }

    pub fn get_content_width(&self) -> usize {
        self.get_terminal_width()
            .saturating_sub(self.margin.total())
            .saturating_sub(self.line_number_gutter_width)
    }

    pub(crate) fn source_line_numbers_enabled(&self) -> bool {
        matches!(
            self.line_numbers,
            Some(options) if options.target == LineNumberTarget::Source
        )
    }

    pub fn validate_horizontal_margins(&self) -> Result<()> {
        let terminal_width = self.get_terminal_width();
        let reserved_width = self
            .margin
            .total()
            .saturating_add(self.line_number_gutter_width);
        if reserved_width >= terminal_width {
            if self.line_number_gutter_width == 0 {
                anyhow::bail!(
                    "Horizontal margins ({} + {}) must be smaller than the output width ({})",
                    self.margin.left,
                    self.margin.right,
                    terminal_width
                );
            }
            anyhow::bail!(
                "Horizontal margins and line-number gutter ({} + {} + {}) must be smaller than the output width ({})",
                self.margin.left,
                self.margin.right,
                self.line_number_gutter_width,
                terminal_width
            );
        }
        Ok(())
    }

    pub(super) fn normalize_theme_settings(&mut self) {
        if self.theme.trim().is_empty() {
            self.theme = "terminal".to_string();
        }

        if let Some(code_theme) = self.code_theme.as_ref()
            && code_theme.trim().is_empty()
        {
            self.code_theme = None;
        }
    }

    pub(super) fn apply_custom_callouts(&mut self) -> Result<()> {
        self.custom_callouts.clear();
        if let Some(raw) = &self.custom_callout {
            self.custom_callouts = parse_custom_callouts(raw)?;
        }
        Ok(())
    }

    pub(super) fn apply_checkbox_overrides(&mut self) -> Result<()> {
        self.checkbox_overrides.clear();
        if self.pretty_checkbox.is_none() {
            return Ok(());
        }
        let Some(raw) = &self.custom_checkbox else {
            return Ok(());
        };
        for entry in raw.split(';') {
            match crate::checkbox_override::CheckboxOverride::parse_entry(entry) {
                Ok(Some((ch, ov))) => {
                    self.checkbox_overrides.insert(ch, ov);
                }
                Ok(None) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    pub(super) fn apply_custom_code_blocks(&mut self) -> Result<()> {
        self.custom_code_blocks.clear();
        self.custom_code_default_icon = None;
        if let Some(raw) = &self.custom_code_block {
            let mut parsed = parse_custom_code_blocks(raw)?;
            if let Some(block) = parsed.remove("default") {
                self.custom_code_default_icon = block.icon;
            }
            self.custom_code_blocks = parsed;
        }
        Ok(())
    }

    pub(super) fn apply_list_markers(&mut self) -> Result<()> {
        // `--custom-list` is a no-op without `--pretty-list`, mirroring `--custom-checkbox`.
        let Some(style) = self.pretty_list else {
            self.list_marker = ListMarkerConfig::default();
            return Ok(());
        };
        self.list_marker = ListMarkerConfig {
            style: Some(style),
            uniform: self.uniform_list_marker.clone(),
            overrides: if let Some(raw) = &self.custom_list {
                ListMarkerConfig::parse_custom_list(raw)?
            } else {
                HashMap::new()
            },
        };
        Ok(())
    }
}
