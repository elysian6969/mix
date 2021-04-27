use std::fmt;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Source {
    Github(String, String),
}

impl Source {
    pub fn parse(string: &str) -> anyhow::Result<Source> {
        let result = string
            .split_once(':')
            .map(|(scheme, rest)| (scheme, rest.split_once('/')));

        match result {
            Some(("github", Some((user, repo)))) => {
                Ok(Source::Github(user.to_owned(), repo.to_owned()))
            }
            Some(("github", None)) => Err(anyhow::anyhow!("missing path")),
            Some((_, _)) => Err(anyhow::anyhow!("unknown source")),
            _ => Err(anyhow::anyhow!("expected source")),
        }
    }
}

impl fmt::Debug for Source {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Source::Github(user, repo) => {
                fmt.write_str("github:")?;
                fmt.write_str(user)?;
                fmt.write_str("/")?;
                fmt.write_str(repo)?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for Source {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Source::Github(user, repo) => {
                fmt.write_str("github:")?;
                fmt.write_str(user)?;
                fmt.write_str("/")?;
                fmt.write_str(repo)?;
            }
        }

        Ok(())
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

impl serde::Serialize for Source {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
