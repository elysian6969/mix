use regex::Regex;
use std::fmt;

/// Group/repository of packages
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Group {
    id: String,
}

impl Group {
    pub fn new(group: impl Into<String>) -> Self {
        Self { id: group.into() }
    }

    pub fn matches(&self, regex: &Regex) -> bool {
        regex.is_match(self.as_str())
    }

    pub fn as_str(&self) -> &str {
        self.id.as_str()
    }
}

impl From<&str> for Group {
    fn from(id: &str) -> Self {
        Self { id: id.into() }
    }
}

impl From<String> for Group {
    fn from(id: String) -> Self {
        Self { id }
    }
}

impl fmt::Debug for Group {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
