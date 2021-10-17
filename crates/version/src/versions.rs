pub use self::iter::Iter;
pub use self::matches::Matches;
pub use self::pairs::Pairs;
pub use self::paths::Paths;
use crate::{Requirement, Version};
use path::{Path, PathBuf};
use std::collections::BTreeMap;
use std::fmt;

mod iter;
mod matches;
mod pairs;
mod paths;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Versions {
    versions: BTreeMap<Version, PathBuf>,
}

impl Versions {
    pub fn new() -> Self {
        let versions = BTreeMap::new();

        Self { versions }
    }

    pub fn insert(&mut self, version: Version, path: impl AsRef<Path>) {
        self.versions.insert(version, path.as_ref().to_path_buf());
    }

    pub fn latest(&self) -> Option<&Version> {
        self.versions.last_key_value().map(|(key, _value)| key)
    }

    pub fn matches<'a>(&'a self, requirement: &'a Requirement) -> Matches<'a> {
        let iter = self.iter();

        Matches { iter, requirement }
    }

    pub fn len(&self) -> usize {
        self.versions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.versions.is_empty()
    }

    pub fn iter(&self) -> Iter<'_> {
        let iter = self.versions.keys();

        Iter { iter }
    }

    pub fn pairs(&self) -> Pairs<'_> {
        let iter = self.versions.iter();

        Pairs { iter }
    }

    pub fn paths(&self) -> Paths<'_> {
        let iter = self.versions.values();

        Paths { iter }
    }
}

impl fmt::Debug for Versions {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_set().entries(self.versions.iter()).finish()
    }
}
