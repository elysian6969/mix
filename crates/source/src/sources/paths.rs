use crate::Source;
use path::{Path, PathBuf};
use std::collections::btree_map;

pub struct Paths<'a> {
    iter: btree_map::Values<'a, Source, PathBuf>,
}

impl<'a> Iterator for Paths<'a> {
    type Item = &'a Path;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().as_path()
    }
}
