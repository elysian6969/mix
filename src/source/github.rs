use crate::config::Config;
use semver::{Version, VersionReq};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use ufmt::derive::uDebug;
use url::Url;

pub struct Repo<'repo> {
    user: &'repo str,
    repo: &'repo str,
}

pub mod metadata {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Tag {
        pub name: Option<String>,
        pub zipball_url: Option<String>,
        pub tarball_url: Option<String>,
        pub commit: Option<Commit>,
    }

    #[derive(Deserialize)]
    pub struct Commit {
        pub sha: Option<String>,
        pub url: Option<String>,
    }
}

impl<'repo> Repo<'repo> {
    pub fn new(user: &'repo str, repo: &'repo str) -> Self {
        Self { user, repo }
    }

    pub async fn tags(&self, config: &Config) -> crate::Result<Tags> {
        let path = config.cache_with(|mut cache| {
            cache.push("github");
            cache.push(self.user);
            cache.push(self.repo);
            cache.push("tags.json");
            cache
        });

        let url = ufmt::uformat!("{}/repos/{}/{}/tags", github_url(), self.user, self.repo)
            .expect("infallible");

        config.client().download(config, &path, url).await?;

        Tags::from_path(path).await
    }
}

#[derive(Debug)]
pub struct Tags {
    tags: BTreeMap<Version, Value>,
}

impl Tags {
    pub async fn from_path(path: impl AsRef<Path>) -> crate::Result<Self> {
        let buffer = fs::read(path.as_ref()).await?;
        let metadata: Vec<metadata::Tag> = serde_json::from_slice(&buffer)?;
        let tags = metadata
            .into_iter()
            .flat_map(|metadata| {
                let version = metadata.name.as_ref()?;
                let version = crate::version::parse(version).ok()?;
                let url = metadata.tarball_url?;
                let url = Url::parse(&url).ok()?;
                let file_name = format!("v{}.tar.gz", version);
                let path = path.as_ref().with_file_name(file_name);

                Some((version, Value { path, url }))
            })
            .collect();

        Ok(Tags { tags })
    }

    pub fn matches<'tags>(
        &'tags self,
        requirement: &'tags VersionReq,
    ) -> impl Iterator<Item = Tag<'tags>> {
        self.tags
            .iter()
            .filter(move |(version, _value)| requirement.matches(&version))
            .map(|(version, value)| Tag { version, value })
    }
}

#[derive(Debug)]
struct Value {
    path: PathBuf,
    url: Url,
}

#[derive(Debug)]
pub struct Tag<'tags> {
    version: &'tags Version,
    value: &'tags Value,
}

impl<'tags> Tag<'tags> {
    pub fn path(&self) -> &Path {
        self.value.path.as_path()
    }

    pub fn url(&self) -> &Url {
        &self.value.url
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub async fn download(&self, config: &Config) -> crate::Result<()> {
        config
            .client()
            .download(config, self.path(), self.url())
            .await
    }
}

pub const fn github_url() -> &'static str {
    "https://api.github.com"
}
