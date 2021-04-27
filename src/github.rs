use crate::version;
use crate::PREFIX;
use semver::Version;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;

pub const API: &str = "https://api.github.com";

#[derive(Debug, Deserialize)]
pub struct Tag {
    pub name: Option<String>,
    pub zipball_url: Option<String>,
    pub tarball_url: Option<String>,
    pub commit: Option<Commit>,
}

#[derive(Debug, Deserialize)]
pub struct Commit {
    pub sha: Option<String>,
    pub url: Option<String>,
}

pub async fn tags(
    http: &reqwest::Client,
    user: &str,
    repository: &str,
) -> anyhow::Result<BTreeMap<Version, String>> {
    let final_path = format!("{PREFIX}/cache/github.com_{user}_{repository}_tags.json");
    let partial_path = format!("{final_path}.part");
    let url = format!("{API}/repos/{user}/{repository}/tags");

    let slice = if Path::new(&final_path).exists() {
        fs::read(&final_path).await?
    } else {
        println!(" -> updating tags");
        let mut buffer = std::io::Cursor::new(Vec::new());
        let mut file = File::create(&partial_path).await?;
        let mut stream = http.get(&url).send().await?.bytes_stream();

        while let Some(bytes) = stream.next().await {
            let slice = &bytes?[..];

            file.write_all(slice).await?;
            buffer.write_all(slice).await?;
        }

        file.flush().await?;
        fs::rename(&partial_path, &final_path).await?;

        buffer.into_inner()
    };

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
