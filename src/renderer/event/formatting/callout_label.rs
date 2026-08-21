use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn should_reserve_callout_padding(&self) -> bool {
        matches!(
            self.config.callout_style.style,
            crate::cli::CalloutStyle::Pretty
        ) && self
            .callout_stack
            .iter()
            .any(|state| matches!(state, CalloutState::Active(_)))
    }

    pub(in crate::renderer::event) fn callout_label_style(
        &self,
        kind: CalloutKind,
        label: &str,
    ) -> AnsiStyle {
        let color = if let Some(custom) = self.config.custom_callouts.get(label) {
            custom
                .color
                .clone()
                .unwrap_or_else(|| self.unknown_callout_color())
        } else {
            self.callout_palette
                .get(&kind)
                .cloned()
                .unwrap_or_else(|| self.theme.text.clone())
        };

        AnsiStyle::new().fg(color.into()).bold()
    }

    pub(in crate::renderer::event) fn unknown_callout_color(&self) -> crate::theme::Color {
        self.callout_palette
            .get(&CalloutKind::Tip)
            .cloned()
            .unwrap_or_else(|| self.theme.text.clone())
    }

    pub(in crate::renderer::event) fn callout_label_text(
        &self,
        label: &str,
        label_override: Option<&str>,
        fold: Option<CalloutFold>,
        icon_spacing: usize,
    ) -> String {
        let base = self.callout_display_label(label, label_override);
        if !self.config.callout_style.icons_enabled() {
            return base;
        }

        let mut text = String::new();
        text.push_str(self.callout_icon_for_label(label));
        if icon_spacing > 0 {
            text.push_str(&" ".repeat(icon_spacing));
        }
        text.push_str(&base);
        if let Some(icon) = self.callout_fold_icon(fold) {
            text.push(' ');
            text.push_str(icon);
        }
        text
    }

    pub(in crate::renderer::event) fn callout_display_label(
        &self,
        label: &str,
        label_override: Option<&str>,
    ) -> String {
        if let Some(label_override) = label_override {
            let trimmed = label_override.trim();
            if !trimmed.is_empty() {
                if self.config.callout_style.uppercase {
                    return trimmed.to_ascii_uppercase();
                }
                return trimmed.to_string();
            }
        }

        self.format_callout_label_case(label)
    }

    pub(in crate::renderer::event) fn callout_fold_icon(
        &self,
        fold: Option<CalloutFold>,
    ) -> Option<&'static str> {
        if !self.config.callout_style.show_icons || !self.config.callout_style.show_fold_icons {
            return None;
        }

        match fold {
            Some(CalloutFold::Expanded) => Some(""),
            Some(CalloutFold::Collapsed) => Some(""),
            None => None,
        }
    }

    pub(in crate::renderer::event) fn format_callout_label_case(&self, label: &str) -> String {
        if self.config.callout_style.uppercase {
            return label.to_ascii_uppercase();
        }

        let lower = label.to_ascii_lowercase();
        if lower == "faq" {
            return "FAQ".to_string();
        }

        let mut chars = lower.chars();
        match chars.next() {
            Some(first) => {
                let mut result = String::new();
                result.push(first.to_ascii_uppercase());
                result.push_str(chars.as_str());
                result
            }
            None => String::new(),
        }
    }

    pub(in crate::renderer::event) fn callout_icon_spacing(&self, label_inside: bool) -> usize {
        if !self.config.callout_style.icons_enabled() {
            return 0;
        }

        if self.config.callout_style.show_simple_icons {
            return 1;
        }

        if matches!(
            self.config.callout_style.style,
            crate::cli::CalloutStyle::Simple
        ) {
            return 2;
        }

        if label_inside { 2 } else { 1 }
    }

    pub(in crate::renderer::event) fn callout_icon_for_label(&self, label: &str) -> &str {
        if let Some(icon) = self
            .config
            .custom_callouts
            .get(label)
            .and_then(|custom| custom.icon.as_deref())
        {
            return icon;
        }

        if self.config.callout_style.show_simple_icons {
            return Self::default_simple_callout_icon_for_label(label)
                .unwrap_or(DEFAULT_UNKNOWN_SIMPLE_CALLOUT_ICON);
        }

        Self::default_callout_icon_for_label(label).unwrap_or(DEFAULT_UNKNOWN_CALLOUT_ICON)
    }

    pub(in crate::renderer::event) fn default_simple_callout_icon_for_label(
        label: &str,
    ) -> Option<&'static str> {
        match label {
            "note" | "seealso" | "abstract" | "summary" | "tldr" | "info" => Some("[i]"),
            "todo" => Some("[ ]"),
            "tip" | "hint" | "example" => Some("[*]"),
            "important" | "warning" | "attention" | "danger" => Some("[!]"),
            "success" | "check" | "done" => Some("[+]"),
            "question" | "help" | "faq" => Some("[?]"),
            "caution" | "failure" | "fail" | "missing" | "error" | "bug" => Some("[x]"),
            "quote" | "cite" => Some("[>]"),
            _ => None,
        }
    }

    pub(in crate::renderer::event) fn default_callout_icon_for_label(
        label: &str,
    ) -> Option<&'static str> {
        match label {
            "note" | "seealso" => Some(""),
            "info" => Some(""),
            "abstract" => Some("󰈚"),
            "summary" | "tldr" => Some(""),
            "example" => Some("󰅍"),
            "todo" => Some("󰅎"),
            "tip" => Some(""),
            "hint" => Some("󰌵"),
            "important" => Some("󰅽"),
            "success" | "check" | "done" => Some(""),
            "question" | "help" => Some(""),
            "faq" => Some("󰠗"),
            "warning" | "caution" | "attention" => Some(""),
            "failure" | "fail" | "missing" | "error" => Some(""),
            "danger" => Some(""),
            "bug" => Some(""),
            "quote" | "cite" => Some(""),
            _ => None,
        }
    }

    pub(in crate::renderer::event) fn render_callout_header(
        &mut self,
        kind: CalloutKind,
        label: &str,
        label_override: Option<&str>,
        fold: Option<CalloutFold>,
    ) {
        let outer_level = self.blockquote_level.saturating_sub(1);
        self.ensure_contextual_blank_line_for_blockquote_level(outer_level);

        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }

        self.push_indent_for_line_start();

        let icon_spacing = self.callout_icon_spacing(false);
        let label_text = self.callout_label_text(label, label_override, fold, icon_spacing);
        let display_label = if matches!(
            self.config.callout_style.style,
            crate::cli::CalloutStyle::Simple
        ) && self.config.callout_style.icons_enabled()
        {
            label_text
        } else {
            format!("[{}]", label_text)
        };
        let label_style = self.callout_label_style(kind, label);
        let styled_label = label_style.apply(&display_label, self.config.no_colors);
        self.output.push_str(&styled_label);

        self.output.push('\n');
        self.push_indent_for_line_start();
        self.output.push('\n');

        if matches!(
            self.config.callout_style.style,
            crate::cli::CalloutStyle::Simple
        ) {
            self.suppress_next_paragraph_break = true;
        }
    }

    pub(in crate::renderer::event) fn maybe_render_callout_header(&mut self) {
        if self.pending_callout_label_override {
            return;
        }
        let mut header = None;
        if let Some(CalloutState::Active(info)) = self.callout_stack.last_mut()
            && !info.header_rendered
        {
            info.header_rendered = true;
            header = Some((
                info.kind,
                info.label.clone(),
                info.label_override.clone(),
                info.fold,
            ));
        }

        if let Some((kind, label, label_override, fold)) = header {
            self.render_callout_header(kind, &label, label_override.as_deref(), fold);
        }
    }

    pub(in crate::renderer::event) fn finalize_pending_callout_label_override(&mut self) -> bool {
        if !self.pending_callout_label_override {
            return false;
        }

        let label_override = self.pending_callout_label_buffer.trim();
        let mut header = None;

        if let Some(CalloutState::Active(info)) = self.callout_stack.last_mut() {
            if !label_override.is_empty() {
                info.label_override = Some(label_override.to_string());
            }

            if !info.header_rendered {
                info.header_rendered = true;
                header = Some((
                    info.kind,
                    info.label.clone(),
                    info.label_override.clone(),
                    info.fold,
                ));
            }
        }

        self.pending_callout_label_override = false;
        self.pending_callout_label_buffer.clear();

        if let Some((kind, label, label_override, fold)) = header {
            self.render_callout_header(kind, &label, label_override.as_deref(), fold);
            return true;
        }

        false
    }
}
