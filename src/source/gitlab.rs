use crate::config::Config;
use semver::{Version, VersionReq};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use ufmt::derive::uDebug;
use url::Url;

pub struct Repo<'base, 'repo> {
    base: &'base str,
    user: &'repo str,
    repo: &'repo str,
}

pub mod metadata {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Tag {
        pub commit: Option<Commit>,
        pub release: Option<Release>,
        pub name: Option<String>,
        pub target: Option<String>,
        pub message: Option<String>,
        pub protected: Option<bool>,
    }

    #[derive(Deserialize)]
    pub struct Commit {
        pub id: Option<String>,
        pub short_id: Option<String>,
        pub title: Option<String>,
        pub created_at: Option<String>,
        #[serde(default)]
        pub parent_ids: Vec<String>,
        pub message: Option<String>,
        pub author_name: Option<String>,
        pub author_email: Option<String>,
        pub authored_date: Option<String>,
        pub comitter_name: Option<String>,
        pub comitter_email: Option<String>,
        pub comitted_date: Option<String>,
    }

    #[derive(Deserialize)]
    pub struct Release {
        pub tag_name: Option<String>,
        pub description: Option<String>,
    }
}

impl<'base, 'repo> Repo<'base, 'repo> {
    pub fn new(base: &'base str, user: &'repo str, repo: &'repo str) -> Self {
        Self { base, user, repo }
    }

    pub async fn tags(&self, config: &Config) -> crate::Result<Tags> {
        let path = config.cache_with(|mut cache| {
            cache.push("gitlab");
            cache.push(self.user);
            cache.push(self.repo);
            cache.push("tags.json");
            cache
        });

        let url = ufmt::uformat!(
            "{}/v4/projects/{}%2F{}/repository/tags",
            self.base,
            self.user,
            self.repo
        )
        .expect("infallible");

        config.client().download(config, &path, url).await?;

        Tags::from_path(self.base, self.user, self.repo, path).await
    }
}

#[derive(Debug)]
pub struct Tags {
    tags: BTreeMap<Version, Value>,
}

impl Tags {
    pub async fn from_path(
        base: &str,
        user: &str,
        repo: &str,
        path: impl AsRef<Path>,
    ) -> crate::Result<Self> {
        let buffer = fs::read(path.as_ref()).await?;
        let metadata: Vec<metadata::Tag> = serde_json::from_slice(&buffer)?;
        let tags = metadata
            .into_iter()
            .flat_map(|metadata| {
                let version = metadata.name?;
                let version = crate::version::parse(&version).ok()?;
                let sha = metadata.target?;
                let url = ufmt::uformat!(
                    "{}/v4/projects/{}%2F{}/repository/archive.tar.gz?sha={}"
                    base,
                    user,
                    repo,
                    sha,
                )
                .ok()?;

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

pub const fn gitlab_url() -> &'static str {
    "https://gitlab.com/api"
}
