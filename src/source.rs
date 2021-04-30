use std::fmt;
use url::Url;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Source {
    url: Url,
}

impl Source {
    pub fn parse(source: &str) -> anyhow::Result<Source> {
        let url = Url::parse(source)?;

        Ok(Source { url })
    }
}

impl<'de> serde::Deserialize<'de> for Source {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error, Unexpected, Visitor};

        struct SourceVisitor;

        impl<'de> Visitor<'de> for SourceVisitor {
            type Value = Source;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing a source")
            }

            fn visit_str<E>(self, string: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Source::parse(string).map_err(|err| {
                    let error_string = format!("{}", err);

                    Error::invalid_value(Unexpected::Str(string), &error_string.as_str())
                })
            }
        }

        deserializer.deserialize_str(SourceVisitor)
    }
}

/*impl serde::Serialize for Source {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}*/
