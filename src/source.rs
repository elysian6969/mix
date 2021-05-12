pub mod github;
pub mod gitlab;
pub mod lexer;
pub mod parser;

use parser::Parser;
use std::fmt;
use ufmt::derive::uDebug;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd, uDebug)]
pub enum Source {
    Github { user: String, repo: String },
    Gitlab { user: String, repo: String },
    Kernel { user: String, repo: String },
    Savannah { repo: String },
    Sourceware { repo: String },
}

impl Source {
    pub fn parse(input: &str) -> crate::Result<Source> {
        let source = Parser::new(input)
            .map_err(|error| {
                let mut buffer = String::from("TODO: implement Error: ");
                let _ = ufmt::uwrite!(buffer, "{:?}", error);

                buffer
            })?
            .parse()
            .map_err(|error| {
                let mut buffer = String::from("TODO: implement Error: ");
                let _ = ufmt::uwrite!(buffer, "{:?}", error);

                buffer
            })?;

        Ok(source)
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
