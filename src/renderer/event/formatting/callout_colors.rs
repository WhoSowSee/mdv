use super::*;

impl<'a> EventRenderer<'a> {
    pub(in crate::renderer::event) fn callout_label_style(
        &self,
        kind: CalloutKind,
        label: &str,
    ) -> AnsiStyle {
        if kind == CalloutKind::Properties {
            return create_style(self.theme, ThemeElement::FrontMatterTitle);
        }
        let (mut color, palette_key) = callout_palette_color(self.theme, kind);
        let priorities = &self.theme.color_priorities;
        let mut source = priorities.element(&format!("callout:palette:{palette_key}"));
        if let Some(common) = self.theme.callout.label.as_ref()
            && priorities.element("callout:label") >= source
        {
            color = common;
            source = priorities.element("callout:label");
        }
        if let Some(custom) = self
            .config
            .custom_callouts
            .get(label)
            .and_then(|value| value.color.as_ref())
            && self
                .config
                .color_priorities
                .custom_color("custom_callout", label)
                >= source
        {
            color = custom;
        }
        create_style(self.theme, ThemeElement::CalloutLabel).fg(color.clone().into())
    }
}
