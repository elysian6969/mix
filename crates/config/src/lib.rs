#![feature(format_args_nl)]

use crate::settings::Settings;
use futures_util::stream::StreamExt;
use mix_id::RepositoryId;
use mix_shell::{write, writeln, AsyncWrite, Shell};
use path::{Path, PathBuf};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::time;
use tokio::time::Duration;
use ubyte::ByteUnit;
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

    /// http client
    http: reqwest::Client,
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

        let http = reqwest::Client::builder()
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            ))
            .build()?;

        Ok(Self(Arc::new(ConfigRef {
            prefix,
            build_prefix,
            cache_prefix,
            repos_prefix,
            shell,
            repositories: settings.repositories,
            http,
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

    pub async fn download_file(&self, path: impl AsRef<Path>, url: impl AsRef<str>) -> Result<()> {
        let path = path.as_ref();

        if path.exists() {
            return Ok(());
        }

        let mut partial = path.to_path_buf();
        let file_name = path.file_name().unwrap_or_else(|| Path::new("<unknown>"));
        let url = url.as_ref();
        let mut downloaded = 0;

        partial.push_str(".partial");

        write!(
            self.shell(),
            "\r\x1b[K > {} {}",
            file_name,
            ByteUnit::Byte(downloaded as u64)
        )?;
        self.shell().flush().await?;

        let mut interval = time::interval(Duration::from_millis(50));
        interval.tick().await;
        let mut destination = File::create(&partial).await?;
        let response = self.0.http.get(url).send().await?;
        let mut stream = response.bytes_stream();

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    write!(self.shell(), "\r\x1b[K > {} {}", file_name, ByteUnit::Byte(downloaded as u64))?;
                    self.shell().flush().await?;
                }
                bytes = stream.next() => if let Some(bytes) = bytes {
                    let bytes = bytes?;
                    let bytes = &bytes[..];

                    downloaded += bytes.len();
                    destination.write_all(bytes).await?;
                } else {
                    break;
                }
            }
        }

        destination.flush().await?;
        partial.rename_async(path).await?;

        writeln!(
            self.shell(),
            "\r\x1b[K > {} {}",
            file_name,
            ByteUnit::Byte(downloaded as u64)
        )?;
        self.shell().flush().await?;

        Ok(())
    }
}
