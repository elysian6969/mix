use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
};

pub struct Config {
    prefix: PathBuf,

    // Sub-directories of ${prefix}
    source: PathBuf,
    build: PathBuf,
    target: PathBuf,
}

impl Config {
    pub fn with_prefix(prefix: impl Into<PathBuf>) -> Self {
        let prefix = prefix.into();
        let source = prefix.clone().join("source");
        let build = prefix.clone().join("build");
        let target = prefix.clone().join("target");

        Self {
            prefix,
            source,
            build,
            target,
        }
    }

    pub fn prefix(&self) -> Cow<Path> {
        Cow::from(&self.prefix)
    }

    pub fn source(&self) -> Cow<Path> {
        Cow::from(&self.source)
    }

    pub fn build(&self) -> Cow<Path> {
        Cow::from(&self.build)
    }

    pub fn target(&self) -> Cow<Path> {
        Cow::from(&self.target)
    }

    pub fn sources(&self) -> anyhow::Result<impl Iterator<Item = PathBuf>> {
        let iter = fs::read_dir(self.source())?
            .flatten()
            .map(|entry| entry.path());

        Ok(iter)
    }

    pub fn targets(&self) -> anyhow::Result<impl Iterator<Item = PathBuf>> {
        let iter = fs::read_dir(self.target())?
            .flatten()
            .map(|entry| entry.path());

        Ok(iter)
    }
}
