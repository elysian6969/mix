use std::borrow::Borrow;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PackageId {
    repr: Box<str>,
}

impl PackageId {
    pub fn new(id: impl Into<Box<str>>) -> Self {
        Self { repr: id.into() }
    }
}

impl From<Box<str>> for PackageId {
    fn from(id: Box<str>) -> Self {
        Self { repr: id }
    }
}

impl From<&str> for PackageId {
    fn from(id: &str) -> Self {
        Self {
            repr: id.to_owned().into_boxed_str(),
        }
    }
}

impl From<&&str> for PackageId {
    fn from(id: &&str) -> Self {
        Self {
            repr: (*id).to_owned().into_boxed_str(),
        }
    }
}

impl From<String> for PackageId {
    fn from(id: String) -> Self {
        Self {
            repr: id.into_boxed_str(),
        }
    }
}

impl From<&String> for PackageId {
    fn from(id: &String) -> Self {
        Self {
            repr: id.into_boxed_str(),
        }
    }
}

impl Borrow<str> for PackageId {
    fn borrow(&self) -> &str {
        &self.repr
    }
}
