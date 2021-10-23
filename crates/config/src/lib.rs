use crate::settings::Settings;
use mix_id::RepositoryId;
use mix_shell::Shell;
use path::{Path, PathBuf};
use std::collections::BTreeMap;
use std::sync::Arc;
use url::Url;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub mod settings;

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

    /// repositories to sync
    repositories: BTreeMap<RepositoryId, Url>,
}

#[derive(Clone, Debug)]
pub struct Config(Arc<ConfigRef>);

impl Config {
    pub async fn new(prefix: impl AsRef<Path>) -> Result<Self> {
        let prefix = prefix.as_ref().to_path_buf();
        let build_prefix = prefix.join("build");
        let cache_prefix = prefix.join("cache");
        let repos_prefix = prefix.join("repos");
        let settings_path = prefix.join("settings.yml");
        let shell = Shell::new();

        if !settings_path.exists_async().await {
            settings_path.write_async("").await?;
        }

        let settings_string = settings_path.read_to_string_async().await?;
        let settings = Settings::parse(settings_string.as_str())?;

        Ok(Self(Arc::new(ConfigRef {
            prefix,
            build_prefix,
            cache_prefix,
            repos_prefix,
            shell,
            repositories: settings.repositories,
        })))
    }

    // system prefix
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

    pub fn repositories(&self) -> &BTreeMap<RepositoryId, Url> {
        &self.0.repositories
    }
}
