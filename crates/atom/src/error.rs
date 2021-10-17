use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    ExpectedPackageId,
    Id(mix_id::Error),
    Version(mix_version::Error),
}

impl From<mix_id::Error> for Error {
    fn from(error: mix_id::Error) -> Self {
        Self::Id(error)
    }
}

impl From<mix_version::Error> for Error {
    fn from(error: mix_version::Error) -> Self {
        Self::Version(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        match &self {
            ExpectedPackageId => fmt.write_str("expected package id")?,
            Id(error) => fmt.write_fmt(format_args!("{}", error))?,
            Version(error) => fmt.write_fmt(format_args!("{}", error))?,
        }

        Ok(())
    }
}

impl error::Error for Error {}
