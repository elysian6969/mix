pub use self::iter::Iter;
use crate::Source;
use path::{Path, PathBuf};
use std::collections::BTreeMap;
use std::fmt;

mod iter;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Sources {
    prefix: PathBuf,
    sources: BTreeMap<Source, PathBuf>,
}

impl Sources {
    pub fn new(prefix: impl AsRef<Path>) -> Self {
        let prefix = prefix.as_ref().to_path_buf();
        let sources = BTreeMap::new();

        Self { prefix, sources }
    }

    pub fn insert(&mut self, source: Source) {
        let path = source.cache(&self.prefix);

        self.sources.insert(source, path);
    }

    pub fn iter(&self) -> Iter<'_> {
        let iter = self.sources.keys();

        Iter { iter }
    }
}

impl fmt::Debug for Sources {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_set().entries(self.sources.iter()).finish()
    }
}
