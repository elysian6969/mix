use mix_shell::Shell;
use path::{Path, PathBuf};
use std::sync::Arc;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
struct ConfigRef {
    /// system prefix
    prefix: PathBuf,

    /// build prefix
    build_prefix: PathBuf,

    /// cache prefix
    cache_prefix: PathBuf,

    /// repos prefix
    repos_prefix: PathBuf,

    /// input/output handler
    shell: Shell,
}

#[derive(Clone, Debug)]
pub struct Config(Arc<ConfigRef>);

impl Config {
    pub fn new(prefix: impl AsRef<Path>) -> Self {
        let prefix = prefix.as_ref().to_path_buf();
        let build_prefix = prefix.join("build");
        let cache_prefix = prefix.join("cache");
        let repos_prefix = prefix.join("repos");
        let shell = Shell::new();

        Self(Arc::new(ConfigRef {
            prefix,
            build_prefix,
            cache_prefix,
            repos_prefix,
            shell,
        }))
    }

    /// system prefix
    pub fn prefix(&self) -> &Path {
        self.0.prefix.as_path()
    }

    /// build prefix
    pub fn build_prefix(&self) -> &Path {
        self.0.build_prefix.as_path()
    }

    /// cache prefix
    pub fn cache_prefix(&self) -> &Path {
        self.0.cache_prefix.as_path()
    }

    /// cache prefix
    pub fn repos_prefix(&self) -> &Path {
        self.0.repos_prefix.as_path()
    }

    /// input/output handler
    pub fn shell(&self) -> &Shell {
        &self.0.shell
    }
}
