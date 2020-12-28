use {
    super::triple::Triple,
    hashbrown::HashMap,
    std::{
        borrow::Cow,
        path::{Path, PathBuf},
    },
};

#[derive(Debug)]
pub struct Config {
    prefix: PathBuf,

    // Sub-directories of ${prefix}
    source: PathBuf,
    build: PathBuf,
    targets: HashMap<Triple, PathBuf>,
}

impl Config {
    pub fn new(prefix: impl Into<PathBuf>, targets: impl IntoIterator<Item = Triple>) -> Self {
        let prefix = prefix.into();
        let source = prefix.clone().join("source");
        let build = prefix.clone().join("build");

        let targets = targets
            .into_iter()
            .map(|triple| (triple, prefix.clone().join(triple.to_string())))
            .collect();

        Self {
            prefix,
            source,
            build,
            targets,
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

    pub fn targets(&self) -> impl Iterator<Item = &Triple> {
        self.targets.keys()
    }

    pub fn target_dirs(&self) -> impl Iterator<Item = (&Triple, &PathBuf)> {
        self.targets.iter()
    }
}
