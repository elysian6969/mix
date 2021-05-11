mod builder;
mod client;

pub use self::builder::Builder;
pub use self::client::Client;

use crate::shell::Shell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use url::Url;

pub struct Config {
    client: Client,
    prefix: PathBuf,
    build: PathBuf,
    cache: PathBuf,
    repositories: BTreeMap<PathBuf, Url>,
    shell: Shell,
}

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
        &self.client
    }

    /// root prefix
    pub fn prefix(&self) -> &Path {
        self.prefix.as_path()
    }

    /// {prefix}/build
    pub fn build(&self) -> &Path {
        self.build.as_path()
    }

    /// {prefix}/cache
    pub fn cache(&self) -> &Path {
        self.cache.as_path()
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
        &self.repositories
    }

    /// shell interaction
    pub fn shell(&self) -> &Shell {
        &self.shell
    }
}
