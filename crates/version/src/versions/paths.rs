use crate::Version;
use path::{Path, PathBuf};
use std::collections::btree_map;

pub struct Paths<'a> {
    pub(crate) iter: btree_map::Values<'a, Version, PathBuf>,
}

impl<'a> Iterator for Paths<'a> {
    type Item = &'a Path;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|path| path.as_path())
    }
}
