use super::spec::Spec;
use super::triple::Triple;
use std::path::{Path, PathBuf};

/// This instance of Watson, lol
#[derive(Debug)]
pub struct Watson {
    root: PathBuf,
}

impl Watson {
    /// New instance with root
    pub fn new(root: &impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    /// Return the root directory of this instance
    pub fn root(&self) -> PathBuf {
        self.root.clone()
    }

    /// Return the source directory of a spec relative to
    /// this instance's root directory
    pub fn source_of(&self, spec: &Spec) -> PathBuf {
        self.root().join("source").join(&spec.package.name)
    }

    /// Return the build directory of a spec relative to this
    /// instance's root directory
    ///
    /// Panics with invalid target triples
    pub fn build_of(&self, spec: &Spec, triple: &Triple) -> PathBuf {
        self.root()
            .join("build")
            .join(triple.to_string().unwrap())
            .join(&spec.package.name)
    }

    /// Return the target directory of a spec relative to this
    /// instance's root directory
    ///
    /// Panics with invalid target triples
    pub fn target_of(&self, spec: &Spec, triple: &Triple) -> PathBuf {
        self.root()
            .join(triple.to_string().unwrap())
            .join(&spec.package.name)
    }

    /// Shorthand for (source_of, build_of, target_of)
    pub fn dirs_of(&self, spec: &Spec, triple: &Triple) -> Dirs {
        Dirs {
            source: self.source_of(&spec),
            build: self.build_of(&spec, &triple),
            target: self.target_of(&spec, &triple),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Dirs {
    pub source: PathBuf,
    pub build: PathBuf,
    pub target: PathBuf,
}
