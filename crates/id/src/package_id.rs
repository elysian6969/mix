use crate::{util, Error, ErrorKind};
use std::borrow::Borrow;
use std::convert::TryFrom;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PackageId {
    repr: Box<str>,
}

impl PackageId {
    pub fn new(id: Box<str>) -> Result<Self, Error> {
        util::validate(&id, ErrorKind::Package)?;

        Ok(Self { repr: id })
    }

    pub fn as_str(&self) -> &str {
        &self.repr
    }
}

impl TryFrom<Box<str>> for PackageId {
    type Error = Error;

    fn try_from(id: Box<str>) -> Result<Self, Self::Error> {
        Self::new(id)
    }
}

impl TryFrom<&str> for PackageId {
    type Error = Error;

    fn try_from(id: &str) -> Result<Self, Self::Error> {
        Self::new(id.to_owned().into_boxed_str())
    }
}

impl TryFrom<&&str> for PackageId {
    type Error = Error;

    fn try_from(id: &&str) -> Result<Self, Self::Error> {
        Self::new((*id).to_owned().into_boxed_str())
    }
}

impl TryFrom<String> for PackageId {
    type Error = Error;

    fn try_from(id: String) -> Result<Self, Self::Error> {
        Self::new(id.into_boxed_str())
    }
}

impl TryFrom<&String> for PackageId {
    type Error = Error;

    fn try_from(id: &String) -> Result<Self, Self::Error> {
        Self::new(id.clone().into_boxed_str())
    }
}

impl Borrow<str> for PackageId {
    fn borrow(&self) -> &str {
        &self.repr
    }
}
