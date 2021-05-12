use crate::config::Config;
use semver::{Version, VersionReq};
use std::collections::BTreeMap;
use std::path::Path;
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
    pub const BASE_URL: &'static str = "https://api.github.com";

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

        let mut url = String::from(Self::BASE_URL);
        let _ = ufmt::uwrite!(url, "/repos/{}/{}/tags", self.user, self.repo);

        fs::create_dir_all(path.parent().expect("infallible")).await?;
        config.client().download(config, &path, url).await?;
        Tags::from_path(path).await
    }
}

#[derive(Debug)]
pub struct Tags {
    tags: BTreeMap<Version, Url>,
}

impl Tags {
    pub async fn from_path(path: impl AsRef<Path>) -> crate::Result<Self> {
        let buffer = fs::read(path).await?;
        let metadata: Vec<metadata::Tag> = serde_json::from_slice(&buffer)?;
        let tags = metadata
            .into_iter()
            .flat_map(|metadata| {
                let version = metadata.name.as_ref()?;
                let version = crate::version::parse(version).ok()?;
                let url = metadata.tarball_url.or(metadata.zipball_url)?;
                let url = Url::parse(&url).ok()?;

                Some((version, url))
            })
            .collect();

        Ok(Tags { tags })
    }

    pub fn matches<'tags>(
        &'tags self,
        requirement: &'tags VersionReq,
    ) -> impl Iterator<Item = (&'tags Version, &'tags Url)> {
        self.tags
            .iter()
            .filter(move |(version, _url)| requirement.matches(&version))
    }
}
