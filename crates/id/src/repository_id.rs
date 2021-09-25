use std::borrow::Borrow;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RepositoryId {
    repr: Box<str>,
}

impl RepositoryId {
    pub fn new(id: impl Into<Box<str>>) -> Self {
        Self { repr: id.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.repr
    }
}

impl From<Box<str>> for RepositoryId {
    fn from(id: Box<str>) -> Self {
        Self { repr: id }
    }
}

impl From<&str> for RepositoryId {
    fn from(id: &str) -> Self {
        Self {
            repr: id.to_owned().into_boxed_str(),
        }
    }
}

impl From<&&str> for RepositoryId {
    fn from(id: &&str) -> Self {
        Self {
            repr: (*id).to_owned().into_boxed_str(),
        }
    }
}

impl From<String> for RepositoryId {
    fn from(id: String) -> Self {
        Self {
            repr: id.into_boxed_str(),
        }
    }
}

impl From<&String> for RepositoryId {
    fn from(id: &String) -> Self {
        Self {
            repr: id.clone().into_boxed_str(),
        }
    }
}

impl Borrow<str> for RepositoryId {
    fn borrow(&self) -> &str {
        &self.repr
    }
}
