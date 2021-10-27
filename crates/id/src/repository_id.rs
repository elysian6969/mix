use crate::{util, Error, ErrorKind};
use regex::Regex;
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::ffi::OsStr;
use std::str::FromStr;
use std::{fmt, mem};

pub(crate) const CORE: &str = "core";

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RepositoryId {
    repr: Box<str>,
}

impl RepositoryId {
    pub const CORE: &'static RepositoryId = &Self::core();

    pub fn new(id: Box<str>) -> Result<Self, Error> {
        util::validate(&id, ErrorKind::Repository)?;

        Ok(Self { repr: id })
    }

    const fn core() -> Self {
        let repr = unsafe { mem::transmute(CORE) };

        RepositoryId { repr }
    }

    pub fn matches(&self, regex: &Regex) -> bool {
        regex.is_match(&self.repr)
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

impl AsRef<str> for RepositoryId {
    fn as_ref(&self) -> &str {
        &self.repr
    }
}

impl AsRef<OsStr> for RepositoryId {
    fn as_ref(&self) -> &OsStr {
        unsafe { &*(&*self.repr as *const str as *const OsStr) }
    }
}

#[cfg(feature = "path")]
impl AsRef<path::Path> for RepositoryId {
    fn as_ref(&self) -> &path::Path {
        unsafe { &*(&*self.repr as *const str as *const path::Path) }
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
