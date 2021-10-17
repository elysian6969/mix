use crate::Version;
use path::{Path, PathBuf};
use std::collections::btree_map;

pub struct Pairs<'a> {
    pub(crate) iter: btree_map::Iter<'a, Version, PathBuf>,
}

impl<'a> Iterator for Pairs<'a> {
    type Item = (&'a Version, &'a Path);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(version, path)| (version, path.as_path()))
    }
}
