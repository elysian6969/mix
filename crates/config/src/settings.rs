use mix_id::RepositoryId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;
use std::{error, fmt, io};
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    #[serde(default, rename = "repos")]
    pub repositories: BTreeMap<RepositoryId, Url>,
}

impl Settings {
    pub fn parse(text: &str) -> Result<Self, Error> {
        let this: Self = match serde_yaml::from_str(text) {
            Ok(this) => this,
            Err(error) => {
                let debug = format!("{:?}", error);

                if debug.contains("EndOfStream") {
                    Self {
                        repositories: BTreeMap::new(),
                    }
                } else {
                    return Err(error.into());
                }
            }
        };

        Ok(this)
    }
}

impl FromStr for Settings {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text)
    }
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Serde(serde_yaml::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

// TODO: Reconstruct error from display of serde_yaml::Error.
impl From<serde_yaml::Error> for Error {
    fn from(error: serde_yaml::Error) -> Self {
        Error::Serde(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        match &self {
            Io(error) => fmt.write_fmt(format_args!("{}", error))?,
            Serde(error) => fmt.write_fmt(format_args!("{}", error))?,
        }

        Ok(())
    }
}

impl error::Error for Error {}
