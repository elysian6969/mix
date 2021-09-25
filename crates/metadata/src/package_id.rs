use std::borrow::Borrow;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RepositoryId {
    repr: String,
}

impl RepositoryId {
    pub fn new(repository_id: impl Into<String>) -> Self {
        Self {
            repr: repository_id.into(),
        }
    }
}

impl From<&str> for RepositoryId {
    fn from(repository_id: &str) -> Self {
        Self {
            repr: repository_id.into(),
        }
    }
}

impl From<String> for RepositoryId {
    fn from(repository_id: String) -> Self {
        Self {
            repr: repository_id,
        }
    }
}

impl Borrow<str> for RepositoryId {
    fn borrow(&self) -> &str {
        &self.repr[..]
    }
}

impl Borrow<String> for RepositoryId {
    fn borrow(&self) -> &String {
        &self.repr
    }
}
