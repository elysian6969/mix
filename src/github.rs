use crate::config::Config;
use crate::version;
use semver::Version;
use serde::Deserialize;
use std::collections::BTreeMap;
use tokio::fs;
use ufmt::derive::uDebug;

pub struct Repo<'repo> {
    user: &'repo str,
    repository: &'repo str,
}

#[derive(Debug, Deserialize, uDebug)]
pub struct Tag {
    pub name: Option<String>,
    pub zipball_url: Option<String>,
    pub tarball_url: Option<String>,
    pub commit: Option<Commit>,
}

#[derive(Debug, Deserialize, uDebug)]
pub struct Commit {
    pub sha: Option<String>,
    pub url: Option<String>,
}

impl<'repo> Repo<'repo> {
    pub const BASE_URL: &'static str = "https://api.github.com";

    pub fn new(user: &'repo str, repository: &'repo str) -> Self {
        Self { user, repository }
    }

    pub async fn tags(&self, config: &Config) -> crate::Result<BTreeMap<Version, Tag>> {
        let path = config.cache_with(|mut cache| {
            cache.push("github");
            cache.push(self.user);
            cache.push(self.repository);
            cache.push("tags.json");
            cache
        });

        let mut url = String::from(Self::BASE_URL);
        let _ = ufmt::uwrite!(url, "/repos/{}/{}/tags", self.user, self.repository);

        fs::create_dir_all(path.parent().expect("infallible")).await?;
        config.client().download(config, &path, url).await?;
        let buffer = fs::read(path).await?;

        let tags: Vec<Tag> = serde_json::from_slice(&buffer)?;
        let tags = tags
            .into_iter()
            .flat_map(|tag| {
                if let Some(ref name) = tag.name {
                    let version = version::parse(name).ok()?;

                    Some((version, tag))
                } else {
                    None
                }
            })
            .collect();

        Ok(tags)
    }
}
