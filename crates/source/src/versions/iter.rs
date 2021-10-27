use super::Entry;
use mix_version::Version;
use path::PathBuf;
use std::collections::btree_map;

pub struct Iter<'a> {
    pub(crate) iter: btree_map::Values<'a, Version, Entry>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Entry;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
