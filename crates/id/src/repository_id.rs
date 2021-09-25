use crate::{util, Error, ErrorKind};
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RepositoryId {
    repr: Box<str>,
}

impl RepositoryId {
    pub fn new(id: Box<str>) -> Result<Self, Error> {
        util::validate(&id, ErrorKind::Repository)?;

        Ok(Self { repr: id })
    }

    pub fn as_str(&self) -> &str {
        &self.repr
    }
}

impl TryFrom<Box<str>> for RepositoryId {
    type Error = Error;

    fn try_from(id: Box<str>) -> Result<Self, Self::Error> {
        Self::new(id)
    }
}

impl TryFrom<&str> for RepositoryId {
    type Error = Error;

    fn try_from(id: &str) -> Result<Self, Self::Error> {
        Self::new(id.to_owned().into_boxed_str())
    }
}

impl TryFrom<&&str> for RepositoryId {
    type Error = Error;

    fn try_from(id: &&str) -> Result<Self, Self::Error> {
        Self::new((*id).to_owned().into_boxed_str())
    }
}

impl TryFrom<String> for RepositoryId {
    type Error = Error;

    fn try_from(id: String) -> Result<Self, Self::Error> {
        Self::new(id.into_boxed_str())
    }
}

impl TryFrom<&String> for RepositoryId {
    type Error = Error;

    fn try_from(id: &String) -> Result<Self, Self::Error> {
        Self::new(id.clone().into_boxed_str())
    }
}

impl FromStr for RepositoryId {
    type Err = Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        Self::new(id.to_owned().into_boxed_str())
    }
}

impl Borrow<str> for RepositoryId {
    fn borrow(&self) -> &str {
        &self.repr
    }
}

impl fmt::Debug for RepositoryId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_fmt(format_args!("{:?}", &self.repr))
    }
}

impl fmt::Display for RepositoryId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&self.repr)
    }
}
