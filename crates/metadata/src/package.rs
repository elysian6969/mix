use regex::Regex;
use std::fmt;

/// Package identifier
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Package {
    id: String,
}

impl Package {
    pub fn new(group: impl Into<String>) -> Self {
        group.into().into()
    }

    pub fn matches(&self, regex: &Regex) -> bool {
        regex.is_match(self.as_str())
    }

    pub fn as_str(&self) -> &str {
        self.id.as_str()
    }
}

impl From<&str> for Package {
    fn from(id: &str) -> Self {
        Self { id: id.into() }
    }
}

impl From<String> for Package {
    fn from(id: String) -> Self {
        Self { id }
    }
}

impl fmt::Debug for Package {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
