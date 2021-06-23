mod builder;
mod client;

pub use self::builder::Builder;
pub use self::client::Client;

use crate::package::Node;
use crate::shell::Shell;
use semver::Version;
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

    pub fn target(&self, target: impl AsRef<Path>) -> PathBuf {
        self.prefix().join(target)
    }

    pub fn target_with<F>(&self, target: impl AsRef<Path>, callback: F) -> PathBuf
    where
        F: FnOnce(PathBuf) -> PathBuf,
    {
        callback(self.target(target))
    }

    pub fn build_dirs(&self, node: &Node, version: &Version) -> (PathBuf, PathBuf) {
        let group = node.group_id.as_str();
        let package = node.package_id.as_str();
        let version = version.to_string();
        let target = "aarch64-unknown-linux-gnu";

        let build_dir = self.build_with(|mut path| {
            path.push(&target);
            path.push(&group);
            path.push(&package);
            path.push(&version);
            path
        });

        let install_dir = self.target_with(&target, |mut path| {
            path.push(&group);
            path.push(&package);
            path.push(&version);
            path
        });

        (build_dir, install_dir)
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
