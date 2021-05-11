use crate::config::Config;
use crate::version;
use semver::Version;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::time;
use tokio::time::Duration;
use tokio_stream::StreamExt;

pub struct Repo<'repo> {
    user: &'repo str,
    repository: &'repo str,
}

#[derive(Deserialize, uDebug)]
pub struct Tag {
    pub name: Option<String>,
    pub zipball_url: Option<String>,
    pub tarball_url: Option<String>,
    pub commit: Option<Commit>,
}

#[derive(Deserialize, uDebug)]
pub struct Commit {
    pub sha: Option<String>,
    pub url: Option<String>,
}

impl<'repo> Repo<'repo> {
    pub const BASE_URL: &str = "https://api.github.com";

    pub async fn tags(&self, confg: &Config) -> crate::Result<BTreeMap<Version, Tag>> {
        let path = config.cache_with(|mut cache| {
            cache.push("github");
            cache.push(self.user);
            cache.push(self.repository);
            cache.push("tags.json");
            cache
        });

        let path = Partial::new(path);
        let mut url = String::from(Self::BASE_URL);
        let _ = ufmt::uwrite!(url, "/repos/{}/{}/tags", self.user, self.repository);

        config.download(url, path).await?;

        println!(" -> parsing tags");

        let tags: Vec<Tag> = serde_json::from_slice(&slice)?;
        let tags = tags
            .into_iter()
            .flat_map(|tag| {
                let name = tag.name?;
                let version = version::parse(&name);

                Some((version, tag.tarball_url?))
            })
            .collect();

        Ok(tags)
    }
}
