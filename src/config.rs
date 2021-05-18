mod builder;
mod client;

pub use self::builder::Builder;
pub use self::client::Client;

use crate::shell::Shell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use url::Url;

crate struct Ref {
    client: Client,
    prefix: PathBuf,
    build: PathBuf,
    cache: PathBuf,
    repositories: BTreeMap<PathBuf, Url>,
    shell: Shell,
}

#[derive(Clone)]
pub struct Config(Arc<Ref>);

impl Config {
    /// config builder
    pub fn builder(prefix: impl Into<PathBuf>) -> Builder {
        Builder {
            prefix: prefix.into(),
            repositories: BTreeMap::new(),
            error: Ok(()),
            user_agent: crate::user_agent().into(),
        }
    }

    /// web request stuff
    pub fn client(&self) -> &Client {
        &self.0.client
    }

    /// root prefix
    pub fn prefix(&self) -> &Path {
        self.0.prefix.as_path()
    }

    /// {prefix}/build
    pub fn build(&self) -> &Path {
        self.0.build.as_path()
    }

    /// helper function to clean up code i guess?
    pub fn build_with<F>(&self, callback: F) -> PathBuf
    where
        F: FnOnce(PathBuf) -> PathBuf,
    {
        callback(self.build().to_path_buf())
    }

    /// {prefix}/cache
    pub fn cache(&self) -> &Path {
        self.0.cache.as_path()
    }

    /// helper function to clean up code i guess?
    pub fn cache_with<F>(&self, callback: F) -> PathBuf
    where
        F: FnOnce(PathBuf) -> PathBuf,
    {
        callback(self.cache().to_path_buf())
    }

    /// map of repositories
    pub fn repositories(&self) -> &BTreeMap<PathBuf, Url> {
        &self.0.repositories
    }

    /// shell interaction
    pub fn shell(&self) -> &Shell {
        &self.0.shell
    }
}
