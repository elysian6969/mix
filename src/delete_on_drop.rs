use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct DeleteOnDrop {
    path: PathBuf,
}

impl DeleteOnDrop {
    pub fn new(path: impl AsRef<Path>) -> DeleteOnDrop {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
}

impl Drop for DeleteOnDrop {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
