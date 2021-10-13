use crate::{util, Error, ErrorKind};
use regex::Regex;
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PackageId {
    repr: Box<str>,
}

impl PackageId {
    pub fn new(id: Box<str>) -> Result<Self, Error> {
        util::validate(&id, ErrorKind::Package)?;

        Ok(Self { repr: id })
    }

    pub fn matches(&self, regex: &Regex) -> bool {
        regex.is_match(&self.repr)
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

impl FromStr for PackageId {
    type Err = Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        Self::new(id.to_owned().into_boxed_str())
    }
}

impl Borrow<str> for PackageId {
    fn borrow(&self) -> &str {
        &self.repr
    }
}

impl fmt::Debug for PackageId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_fmt(format_args!("{:?}", &self.repr))
    }
}

impl fmt::Display for PackageId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&self.repr)
    }
}
