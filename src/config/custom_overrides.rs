use super::Config;
use crate::theme::ColorSource;
use anyhow::Result;
use std::collections::BTreeMap;

mod parsing;

type Fields = BTreeMap<String, Option<String>>;
type Entries = BTreeMap<String, Option<Fields>>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CustomSetting {
    Callout,
    CodeBlock,
    Checkbox,
    List,
}

impl CustomSetting {
    pub(crate) const ALL: [Self; 4] = [Self::Callout, Self::CodeBlock, Self::Checkbox, Self::List];

    pub(crate) fn key(self) -> &'static str {
        match self {
            Self::Callout => "custom_callout",
            Self::CodeBlock => "custom_code_block",
            Self::Checkbox => "custom_checkbox",
            Self::List => "custom_list",
        }
    }

    pub(crate) fn value(self, config: &mut Config) -> &mut Option<String> {
        match self {
            Self::Callout => &mut config.custom_callout,
            Self::CodeBlock => &mut config.custom_code_block,
            Self::Checkbox => &mut config.custom_checkbox,
            Self::List => &mut config.custom_list,
        }
    }
}

impl Config {
    pub(super) fn initialize_custom_overrides(&mut self) -> Result<()> {
        for setting in CustomSetting::ALL {
            if let Some(raw) = setting.value(self).take() {
                self.merge_custom_override(setting, Some(&raw), ColorSource::Config)?;
            }
        }
        Ok(())
    }

    pub(crate) fn merge_custom_override(
        &mut self,
        setting: CustomSetting,
        raw: Option<&str>,
        source: ColorSource,
    ) -> Result<()> {
        let patch = parsing::parse(setting, raw)?;
        let Some(patch) = patch else {
            *setting.value(self) = None;
            self.color_priorities.clear_custom_colors(setting.key());
            return Ok(());
        };
        let mut entries =
            parsing::parse(setting, setting.value(self).as_deref())?.unwrap_or_default();
        for (name, fields) in patch {
            let Some(fields) = fields else {
                entries.remove(&name);
                self.color_priorities
                    .set_custom_color(setting.key(), &name, None);
                continue;
            };
            let target = entries
                .entry(name.clone())
                .or_default()
                .get_or_insert_default();
            for (field, value) in fields {
                if field == "color" {
                    self.color_priorities.set_custom_color(
                        setting.key(),
                        &name,
                        value.as_ref().map(|_| source),
                    );
                }
                if value.is_some() {
                    target.insert(field, value);
                } else {
                    target.remove(&field);
                }
            }
            if target.is_empty() {
                entries.remove(&name);
            }
        }
        *setting.value(self) = serialize(setting, entries);
        Ok(())
    }
}

fn serialize(setting: CustomSetting, entries: Entries) -> Option<String> {
    let entries = entries
        .into_iter()
        .filter_map(|(name, fields)| {
            let fields = fields?;
            let options = if matches!(setting, CustomSetting::Callout | CustomSetting::CodeBlock) {
                fields
                    .into_iter()
                    .filter_map(|(key, value)| value.map(|value| format!("{key}={value}")))
                    .collect::<Vec<_>>()
                    .join(",")
            } else {
                match (
                    fields.get("icon").and_then(Option::as_deref),
                    fields.get("color").and_then(Option::as_deref),
                ) {
                    (Some(icon), color) => format!("{icon}:{}", color.unwrap_or("")),
                    (None, Some(color)) => color.to_string(),
                    (None, None) => return None,
                }
            };
            (!options.is_empty()).then(|| format!("{name}:{options}"))
        })
        .collect::<Vec<_>>();
    (!entries.is_empty()).then(|| entries.join(";"))
}
