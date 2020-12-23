use std::{fs, path::PathBuf};

pub struct DeleteOnDrop {
    path: PathBuf,
}

impl DeleteOnDrop {
    pub fn new(path: impl Into<PathBuf>) -> DeleteOnDrop {
        Self { path: path.into() }
    }
}

impl Drop for DeleteOnDrop {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
