use super::{Config, ConfigSchema};
use serde::de::value::{MapAccessDeserializer, StringDeserializer};
use serde::de::{DeserializeSeed, Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;

pub(crate) const REMOVED_COLOR_SETTING: &str =
    "the 'no_colors' setting was removed; use 'color: auto', 'color: always', or 'color: never'";

impl<'de> Deserialize<'de> for Config {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(ConfigVisitor)
    }
}

struct ConfigVisitor;

impl<'de> Visitor<'de> for ConfigVisitor {
    type Value = Config;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a configuration mapping")
    }

    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Config, A::Error> {
        ConfigSchema::deserialize(MapAccessDeserializer::new(ConfigMap(map)))
    }
}

struct ConfigMap<A>(A);

impl<'de, A: MapAccess<'de>> MapAccess<'de> for ConfigMap<A> {
    type Error = A::Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, A::Error> {
        let Some(key) = self.0.next_key::<String>()? else {
            return Ok(None);
        };
        if key == "no_colors" {
            return Err(A::Error::custom(REMOVED_COLOR_SETTING));
        }
        seed.deserialize(StringDeserializer::<A::Error>::new(key))
            .map(Some)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, A::Error> {
        self.0.next_value_seed(seed)
    }

    fn size_hint(&self) -> Option<usize> {
        self.0.size_hint()
    }
}
