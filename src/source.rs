use std::fmt;
use ufmt::derive::uDebug;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd, uDebug)]
pub enum Source {
    Github { user: String, repository: String },
}

impl Source {
    pub fn parse(source: &str) -> crate::Result<Source> {
        parse(source)
    }
}

fn parse(source: &str) -> crate::Result<Source> {
    let source = source.split_once(':');

    match source {
        Some(("github", path)) => parse_github(path),
        _ => Err("invalid")?,
    }
}

fn parse_github(path: &str) -> crate::Result<Source> {
    match path.split_once('/') {
        Some((user, repository)) => Ok(Source::Github {
            user: user.into(),
            repository: repository.into(),
        }),
        _ => Err("invalid")?,
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
