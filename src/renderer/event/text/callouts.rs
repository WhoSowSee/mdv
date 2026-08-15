use super::*;

impl<'a> EventRenderer<'a> {
    pub(super) fn split_custom_task_marker_prefix<'b>(
        &self,
        text: &'b str,
    ) -> Option<(&'b str, &'b str)> {
        let after_open = text.strip_prefix('[')?;
        let state = after_open.chars().next()?;
        let after_state = &after_open[state.len_utf8()..];
        let remainder = after_state.strip_prefix(']')?;
        let marker_end = text.len() - remainder.len();
        Some((&text[..marker_end], remainder))
    }

    pub(in crate::renderer::event) fn is_custom_task_marker(&self, text: &str) -> bool {
        matches!(self.split_custom_task_marker_prefix(text), Some((_, "")))
    }

    pub(super) fn parse_callout_marker(text: &str) -> Option<CalloutMarker> {
        let trimmed = text.trim_start();
        if !trimmed.starts_with("[!") {
            return None;
        }

        let closing = trimmed.find(']')?;
        if closing < 2 {
            return None;
        }

        let kind_raw = trimmed[2..closing].trim();
        if kind_raw.is_empty() || !Self::is_valid_callout_kind(kind_raw) {
            return None;
        }

        let (kind, label) = Self::resolve_callout_kind(kind_raw);
        let mut rest = &trimmed[closing + 1..];
        let mut fold = None;

        if let Some(first) = rest.chars().next()
            && matches!(first, '+' | '-')
        {
            fold = Some(match first {
                '+' => CalloutFold::Expanded,
                '-' => CalloutFold::Collapsed,
                _ => unreachable!(),
            });
            rest = &rest[first.len_utf8()..];
        }

        if rest.is_empty() {
            return Some(CalloutMarker {
                kind,
                label,
                label_override: None,
                fold,
                trailing: None,
                allow_label_override: true,
                suppress_paragraph_break: false,
            });
        }

        let starts_with_ws = rest
            .chars()
            .next()
            .map(|ch| ch.is_whitespace())
            .unwrap_or(false);
        if starts_with_ws {
            let label_override_raw = rest.trim();
            if label_override_raw.is_empty() {
                return Some(CalloutMarker {
                    kind,
                    label,
                    label_override: None,
                    fold,
                    trailing: None,
                    allow_label_override: false,
                    suppress_paragraph_break: false,
                });
            }
            return Some(CalloutMarker {
                kind,
                label,
                label_override: Some(label_override_raw.to_string()),
                fold,
                trailing: None,
                allow_label_override: false,
                suppress_paragraph_break: false,
            });
        }

        Some(CalloutMarker {
            kind,
            label,
            label_override: None,
            fold,
            trailing: None,
            allow_label_override: false,
            suppress_paragraph_break: true,
        })
    }

    pub(in crate::renderer::event) fn is_callout_marker_text(text: &str) -> bool {
        Self::parse_callout_marker(text).is_some()
    }

    pub(super) fn evaluate_callout_buffer(buffer: &str) -> CalloutBufferEval {
        let trimmed = buffer.trim_start();
        if !trimmed.starts_with('[') {
            return CalloutBufferEval::NotCallout;
        }

        if trimmed.len() >= 2 && !trimmed.starts_with("[!") {
            return CalloutBufferEval::NotCallout;
        }

        if trimmed.contains(']') {
            return match Self::parse_callout_marker(buffer) {
                Some(marker) => CalloutBufferEval::Callout(marker),
                None => CalloutBufferEval::NotCallout,
            };
        }

        if trimmed.len() > CALLOUT_BUFFER_LIMIT {
            return CalloutBufferEval::NotCallout;
        }

        CalloutBufferEval::Pending
    }

    pub(super) fn apply_callout_buffer_evaluation(
        state: &mut CalloutState,
        evaluation: CalloutBufferEval,
        buffer: &str,
    ) -> CalloutDecision {
        match evaluation {
            CalloutBufferEval::Callout(marker) => {
                let has_label_override = marker
                    .label_override
                    .as_ref()
                    .map(|text| !text.trim().is_empty())
                    .unwrap_or(false);
                let defer_label_override = marker.allow_label_override && !has_label_override;
                let info = CalloutInfo {
                    kind: marker.kind,
                    label: marker.label.clone(),
                    label_override: marker.label_override.clone(),
                    fold: marker.fold,
                    header_rendered: !defer_label_override,
                    min_heading_indent: None,
                    inline_link_counter: 0,
                    inline_links: Vec::new(),
                };
                *state = CalloutState::Active(info);
                if defer_label_override {
                    CalloutDecision::AwaitLabelOverride
                } else {
                    CalloutDecision::RenderHeader {
                        kind: marker.kind,
                        label: marker.label,
                        label_override: marker.label_override,
                        fold: marker.fold,
                        trailing: marker.trailing,
                        suppress_paragraph_break: marker.suppress_paragraph_break,
                    }
                }
            }
            CalloutBufferEval::NotCallout => {
                *state = CalloutState::None;
                CalloutDecision::FlushBuffer(buffer.to_string())
            }
            CalloutBufferEval::Pending => CalloutDecision::Pending,
        }
    }

    pub(super) fn is_valid_callout_kind(kind: &str) -> bool {
        kind.chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    }

    pub(super) fn resolve_callout_kind(raw: &str) -> (CalloutKind, String) {
        let lower = raw.trim().to_ascii_lowercase();
        let kind = match lower.as_str() {
            "note" | "seealso" => CalloutKind::Note,
            "abstract" | "summary" | "tldr" => CalloutKind::Abstract,
            "info" => CalloutKind::Info,
            "todo" => CalloutKind::Todo,
            "tip" | "hint" | "important" => CalloutKind::Tip,
            "success" | "check" | "done" => CalloutKind::Success,
            "question" | "help" | "faq" => CalloutKind::Question,
            "warning" | "caution" | "attention" => CalloutKind::Warning,
            "failure" | "fail" | "missing" => CalloutKind::Failure,
            "danger" | "error" => CalloutKind::Danger,
            "bug" => CalloutKind::Bug,
            "example" => CalloutKind::Example,
            "quote" | "cite" => CalloutKind::Quote,
            _ => CalloutKind::Tip,
        };

        (kind, lower)
    }
}
