use crate::{util, Error, ErrorKind};
use std::borrow::Borrow;
use std::convert::TryFrom;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

impl Borrow<str> for RepositoryId {
    fn borrow(&self) -> &str {
        &self.repr
    }
}
