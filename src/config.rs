use super::index::Index;
use super::triple::Triple;
use reqwest::Client;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;

pub const DISTRO: &str = "tiramisu";
pub const DISTRO_VERSION: &str = "0.0.6";
pub const PREFIX: &str = "/tiramisu";
pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub struct Builder {
    root: PathBuf,
    targets: BTreeSet<Triple>,
}

impl Builder {
    pub fn target(mut self, triple: Triple) -> Self {
        self.targets.insert(triple);
        self
    }

    pub fn build(self) -> anyhow::Result<Config> {
        let Self { root, targets } = self;

        let build = root.join("build");
        let cache = root.join("cache");
        let source = root.join("source");

        let client = Client::builder().user_agent(USER_AGENT).build()?;

        let index = RefCell::new(Index::from_file(root.join("packages.yml")));

        let targets: BTreeMap<_, _> = targets
            .into_iter()
            .map(|target| (target, root.join(target.to_string())))
            .collect();

        Ok(Config {
            build,
            cache,
            client,
            index,
            root,
            source,
            targets,
        })
    }
}

pub struct Context {
    prefix: PathBuf,
}

#[derive(Debug)]
pub struct Config {
    build: PathBuf,
    cache: PathBuf,
    client: Client,
    index: RefCell<Index>,
    root: PathBuf,
    source: PathBuf,
    targets: BTreeMap<Triple, PathBuf>,
}

impl Config {
    pub fn builder(root: impl AsRef<Path>) -> Builder {
        Builder {
            root: root.as_ref().to_path_buf(),
            targets: BTreeSet::new(),
        }
    }

    pub fn build(&self) -> &Path {
        self.build.as_path()
    }

    pub fn cache(&self) -> &Path {
        self.cache.as_path()
    }

    pub fn cache_with(&self, path: impl AsRef<Path>) -> PathBuf {
        self.cache().join(path)
    }

    pub async fn fetch_cached(
        &self,
        file_name: impl AsRef<Path>,
        url: impl AsRef<str>,
    ) -> anyhow::Result<()> {
        println!(
            "Config::fetch_cached(file_name={}, url={})",
            file_name.as_ref().display(),
            url.as_ref()
        );

        let path = self.cache_with(file_name);

        println!("path: {}", path.display());

        if !path.exists() {
            let partial_path = with_partial(&path);
            println!("partial_path: {}", partial_path.display());

            let result = fs::create_dir_all(partial_path.parent().unwrap()).await;

            println!("fs::create_dir_all: {:?}", result);

            let mut file = File::create(&partial_path).await?;
            let mut stream = self.client.get(url.as_ref()).send().await?.bytes_stream();

            while let Some(slice) = stream.next().await {
                file.write_all(&slice?[..]).await?;
            }

            file.flush().await?;

            fs::rename(&partial_path, &path).await?;
        }

        Ok(())
    }

    pub fn index(&self) -> &RefCell<Index> {
        &self.index
    }

    pub fn root(&self) -> &Path {
        self.root.as_path()
    }

    pub fn source(&self) -> &Path {
        self.source.as_path()
    }

    pub fn targets(&self) -> &BTreeMap<Triple, PathBuf> {
        &self.targets
    }
}

fn with_partial(path: impl AsRef<Path>) -> PathBuf {
    let mut file_name = path.as_ref().file_name().unwrap().to_os_string();

    file_name.push(".partial");

    path.as_ref().with_file_name(file_name)
}
