use std::path::{Path, PathBuf};

pub struct Partial {
    whole: PathBuf,
    partial: PathBuf,
}

impl Partial {
    pub const EXTENSION: &'static str = "partial";

    pub fn new(path: impl Into<PathBuf>) -> Self {
        let whole = path.into();
        let mut partial = whole.clone();

        if let Some(extension) = partial.extension() {
            let mut extension = PathBuf::from(extension);

            extension.set_extension(Self::EXTENSION);
            partial.set_extension(extension);
        } else {
            partial.set_extension(Self::EXTENSION);
        }

        Self { whole, partial }
    }

    pub fn whole(&self) -> &Path {
        self.whole.as_path()
    }

    pub fn partial(&self) -> &Path {
        self.partial.as_path()
    }
}
