use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum BlockElement {
    Paragraph,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    CodeBlock,
    DisplayMath,
    Table,
    HorizontalRule,
    UnorderedList,
    OrderedList,
    TaskList,
    Blockquote,
    Callout,
    DefinitionList,
    InlineReferences,
    EndReferences,
    AttachedFootnotes,
    Endnotes,
}

impl BlockElement {
    const fn default_spacing(self) -> BlockSpacing {
        match self {
            Self::Paragraph => BlockSpacing { top: 0, bottom: 1 },
            Self::EndReferences => BlockSpacing { top: 2, bottom: 1 },
            _ => BlockSpacing { top: 1, bottom: 1 },
        }
    }
}

impl fmt::Display for BlockElement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Paragraph => "paragraph",
            Self::H1 => "h1",
            Self::H2 => "h2",
            Self::H3 => "h3",
            Self::H4 => "h4",
            Self::H5 => "h5",
            Self::H6 => "h6",
            Self::CodeBlock => "code-block",
            Self::DisplayMath => "display-math",
            Self::Table => "table",
            Self::HorizontalRule => "horizontal-rule",
            Self::UnorderedList => "unordered-list",
            Self::OrderedList => "ordered-list",
            Self::TaskList => "task-list",
            Self::Blockquote => "blockquote",
            Self::Callout => "callout",
            Self::DefinitionList => "definition-list",
            Self::InlineReferences => "inline-references",
            Self::EndReferences => "end-references",
            Self::AttachedFootnotes => "attached-footnotes",
            Self::Endnotes => "endnotes",
        })
    }
}

impl FromStr for BlockElement {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "paragraph" => Ok(Self::Paragraph),
            "h1" => Ok(Self::H1),
            "h2" => Ok(Self::H2),
            "h3" => Ok(Self::H3),
            "h4" => Ok(Self::H4),
            "h5" => Ok(Self::H5),
            "h6" => Ok(Self::H6),
            "code-block" => Ok(Self::CodeBlock),
            "display-math" => Ok(Self::DisplayMath),
            "table" => Ok(Self::Table),
            "horizontal-rule" => Ok(Self::HorizontalRule),
            "unordered-list" => Ok(Self::UnorderedList),
            "ordered-list" => Ok(Self::OrderedList),
            "task-list" => Ok(Self::TaskList),
            "blockquote" => Ok(Self::Blockquote),
            "callout" => Ok(Self::Callout),
            "definition-list" => Ok(Self::DefinitionList),
            "inline-references" => Ok(Self::InlineReferences),
            "end-references" => Ok(Self::EndReferences),
            "attached-footnotes" => Ok(Self::AttachedFootnotes),
            "endnotes" => Ok(Self::Endnotes),
            unknown => Err(format!("unknown block spacing element '{unknown}'")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockSpacing {
    pub(crate) top: usize,
    pub(crate) bottom: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct PartialBlockSpacing {
    top: Option<usize>,
    bottom: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockSpacingOverrides {
    entries: BTreeMap<BlockElement, PartialBlockSpacing>,
}

impl BlockSpacingOverrides {
    pub(crate) fn spacing(&self, element: BlockElement) -> BlockSpacing {
        let mut spacing = element.default_spacing();
        if let Some(overrides) = self.entries.get(&element) {
            if let Some(top) = overrides.top {
                spacing.top = top;
            }
            if let Some(bottom) = overrides.bottom {
                spacing.bottom = bottom;
            }
        }
        spacing
    }

    pub(crate) fn merge(&mut self, other: &Self) {
        for (&element, overrides) in &other.entries {
            let current = self.entries.entry(element).or_default();
            if let Some(top) = overrides.top {
                current.top = Some(top);
            }
            if let Some(bottom) = overrides.bottom {
                current.bottom = Some(bottom);
            }
        }
    }
}

impl FromStr for BlockSpacingOverrides {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut entries = BTreeMap::new();

        for raw_entry in value.split(';') {
            let entry = raw_entry.trim();
            let (raw_element, raw_options) = entry
                .split_once(':')
                .ok_or_else(|| format!("invalid block spacing entry '{entry}'"))?;
            let element = raw_element.parse::<BlockElement>()?;
            if entries.contains_key(&element) {
                return Err(format!("duplicate block spacing element '{element}'"));
            }

            let raw_options = raw_options.trim();
            if raw_options.is_empty() {
                return Err(format!("block spacing element '{element}' has no options"));
            }

            let mut spacing = PartialBlockSpacing::default();
            for raw_option in raw_options.split(',') {
                let option = raw_option.trim();
                let (raw_name, raw_value) = option
                    .split_once('=')
                    .ok_or_else(|| format!("invalid block spacing option '{option}'"))?;
                let name = raw_name.trim();
                let raw_value = raw_value.trim();
                let count = raw_value.parse::<usize>().map_err(|_| {
                    format!("block spacing '{name}' must be a non-negative integer")
                })?;

                match name {
                    "top" if spacing.top.is_none() => spacing.top = Some(count),
                    "bottom" if spacing.bottom.is_none() => spacing.bottom = Some(count),
                    "top" | "bottom" => {
                        return Err(format!(
                            "duplicate block spacing option '{name}' for '{element}'"
                        ));
                    }
                    unknown => return Err(format!("unknown block spacing option '{unknown}'")),
                }
            }

            entries.insert(element, spacing);
        }

        Ok(Self { entries })
    }
}

impl fmt::Display for BlockSpacingOverrides {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (entry_index, (element, spacing)) in self.entries.iter().enumerate() {
            if entry_index > 0 {
                formatter.write_str(";")?;
            }
            write!(formatter, "{element}:")?;
            match (spacing.top, spacing.bottom) {
                (Some(top), Some(bottom)) => write!(formatter, "top={top},bottom={bottom}")?,
                (Some(top), None) => write!(formatter, "top={top}")?,
                (None, Some(bottom)) => write!(formatter, "bottom={bottom}")?,
                (None, None) => unreachable!(),
            }
        }
        Ok(())
    }
}

impl Serialize for BlockSpacingOverrides {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.entries.is_empty() {
            serializer.serialize_none()
        } else {
            serializer.serialize_str(&self.to_string())
        }
    }
}

impl<'de> Deserialize<'de> for BlockSpacingOverrides {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Input {
            String(String),
            Mapping(BTreeMap<String, PartialBlockSpacing>),
        }

        match Option::<Input>::deserialize(deserializer)? {
            None => Ok(Self::default()),
            Some(Input::String(value)) => value.parse().map_err(de::Error::custom),
            Some(Input::Mapping(values)) => {
                let mut entries = BTreeMap::new();
                for (name, spacing) in values {
                    let element = name.parse::<BlockElement>().map_err(de::Error::custom)?;
                    if spacing.top.is_none() && spacing.bottom.is_none() {
                        return Err(de::Error::custom(format!(
                            "block spacing element '{element}' has no options"
                        )));
                    }
                    if entries.insert(element, spacing).is_some() {
                        return Err(de::Error::custom(format!(
                            "duplicate block spacing element '{element}'"
                        )));
                    }
                }
                Ok(Self { entries })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_overrides_round_trip_through_yaml() {
        let serialized = serde_yaml::to_string(&BlockSpacingOverrides::default())
            .expect("serialize default block spacing");
        assert_eq!(serialized, "null\n");

        let parsed: BlockSpacingOverrides =
            serde_yaml::from_str(&serialized).expect("deserialize default block spacing");

        assert_eq!(parsed, BlockSpacingOverrides::default());
    }
}
