use super::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HorizontalMargins {
    pub left: usize,
    pub right: usize,
}

impl HorizontalMargins {
    fn parse(raw: &str) -> Result<Self, String> {
        let input = raw.trim();
        if input.is_empty() {
            return Err("Horizontal margins cannot be empty.".to_string());
        }

        let mut margins = Self::default();
        let mut has_left = false;
        let mut has_right = false;

        for pair in input.split(';') {
            let pair = pair.trim();
            let Some((side, columns)) = pair.split_once(':') else {
                return Err(format!(
                    "Invalid horizontal margin '{}'. Expected '<side>:<columns>'.",
                    pair
                ));
            };
            let side = side.trim().to_ascii_lowercase();
            let columns = columns.trim().parse::<usize>().map_err(|_| {
                format!(
                    "Invalid horizontal margin value '{}'. Expected a non-negative integer.",
                    columns.trim()
                )
            })?;

            match side.as_str() {
                "left" if !has_left => {
                    margins.left = columns;
                    has_left = true;
                }
                "right" if !has_right => {
                    margins.right = columns;
                    has_right = true;
                }
                "left" | "right" => {
                    return Err(format!(
                        "Horizontal margin '{}' is defined more than once.",
                        side
                    ));
                }
                _ => {
                    return Err(format!(
                        "Unknown horizontal margin '{}'. Expected 'left' or 'right'.",
                        side
                    ));
                }
            }
        }

        Ok(margins)
    }

    pub fn total(self) -> usize {
        self.left.saturating_add(self.right)
    }
}

impl fmt::Display for HorizontalMargins {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "left:{};right:{}", self.left, self.right)
    }
}

impl serde::Serialize for HorizontalMargins {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if *self == Self::default() {
            serializer.serialize_none()
        } else {
            serializer.serialize_str(&self.to_string())
        }
    }
}

impl<'de> serde::Deserialize<'de> for HorizontalMargins {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <Option<String> as serde::Deserialize>::deserialize(deserializer)?;
        match value {
            Some(value) => Self::parse(&value).map_err(serde::de::Error::custom),
            None => Ok(Self::default()),
        }
    }
}

impl TryFrom<String> for HorizontalMargins {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<HorizontalMargins> for String {
    fn from(value: HorizontalMargins) -> Self {
        value.to_string()
    }
}

pub(super) fn parse_horizontal_margins(value: &str) -> Result<HorizontalMargins, String> {
    HorizontalMargins::parse(value)
}
