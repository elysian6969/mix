use mix_version::{Requirement, Version};
use path::PathBuf;
use url::Url;

#[derive(Clone, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct Entry {
    pub path: PathBuf,
    pub url: Url,
    pub version: Version,
}

impl Entry {
    pub fn matches(&self, requirement: &Requirement) -> bool {
        self.version.matches(requirement)
    }
}
