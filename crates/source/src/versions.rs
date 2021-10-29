pub use self::entry::Entry;
pub use self::iter::Iter;
pub use self::matches::Matches;
use mix_version::{Requirement, Version};
use std::collections::BTreeMap;
use std::fmt;

mod entry;
mod iter;
mod matches;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Versions {
    pub(crate) versions: BTreeMap<Version, Entry>,
}

impl Versions {
    pub fn new() -> Self {
        let versions = BTreeMap::new();

        Self { versions }
    }

    pub fn insert(&mut self, entry: Entry) {
        let version = entry.version.clone();

        self.versions.insert(version, entry);
    }

    pub fn latest(&self) -> Option<&Entry> {
        self.versions
            .last_key_value()
            .map(|(_version, entry)| entry)
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
        let iter = self.versions.values();

        Iter { iter }
    }
}

impl Default for Versions {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Versions {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_set().entries(self.versions.iter()).finish()
    }
}
