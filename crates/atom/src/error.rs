use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    ExpectedPackageId,
    Id(milk_id::Error),
    Semver(semver::Error),
}

impl From<milk_id::Error> for Error {
    fn from(error: milk_id::Error) -> Self {
        Self::Id(error)
    }
}

impl From<semver::Error> for Error {
    fn from(error: semver::Error) -> Self {
        Self::Semver(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        match &self {
            ExpectedPackageId => fmt.write_str("expected package id")?,
            Id(error) => fmt.write_fmt(format_args!("{}", error))?,
            Semver(error) => fmt.write_fmt(format_args!("{}", error))?,
        }

        Ok(())
    }
}

impl error::Error for Error {}
