use {
    super::client::Client,
    hashbrown::HashMap,
    semver::{AlphaNumeric, Version},
    serde::Deserialize,
    std::{collections::BTreeMap, path::PathBuf},
    url::Url,
};

#[derive(Debug, Deserialize)]
pub struct Tag {
    pub name: String,
    pub zipball_url: Url,
    pub tarball_url: Url,
    pub commit: Commit,
}

#[derive(Debug, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub url: Url,
}

pub fn parse_hyphened(input: &str) -> Option<(u64, Option<&str>)> {
    let mut parts = input.splitn(2, |c: char| c == '_' || c == '-');

    let version = parts.next()?.parse().ok()?;
    let build = parts.next();

    Some((version, build))
}

pub fn parse_version(input: &str) -> Version {
    let mut trimmed = input.trim_start_matches(|c: char| !c.is_ascii_digit());

    if trimmed.is_empty() {
        trimmed = input;
    }

    let mut parts = trimmed.splitn(3, '.').flat_map(parse_hyphened);
    let version = match (parts.next(), parts.next(), parts.next()) {
        // 1.2.3-build & 1.2.3_build
        (Some((major, None)), Some((minor, None)), Some((patch, Some(pre)))) => Version {
            major,
            minor,
            patch,
            pre: vec![AlphaNumeric(String::from(pre))],
            build: Vec::new(),
        },
        // 1.2.3
        (Some((major, None)), Some((minor, None)), Some((patch, None))) => Version {
            major,
            minor,
            patch,
            pre: Vec::new(),
            build: Vec::new(),
        },
        // 1.2-build && 1.2_build
        (Some((major, None)), Some((minor, Some(pre))), None) => Version {
            major,
            minor,
            patch: 0,
            pre: vec![AlphaNumeric(String::from(pre))],
            build: Vec::new(),
        },
        // 1.2
        (Some((major, None)), Some((minor, None)), None) => Version {
            major,
            minor,
            patch: 0,
            pre: Vec::new(),
            build: Vec::new(),
        },
        // 1-build && 1_build
        (Some((major, Some(pre))), None, None) => Version {
            major,
            minor: 0,
            patch: 0,
            pre: vec![AlphaNumeric(String::from(pre))],
            build: Vec::new(),
        },
        // 1
        (Some((major, None)), None, None) => Version {
            major,
            minor: 0,
            patch: 0,
            pre: Vec::new(),
            build: Vec::new(),
        },
        _ => Version {
            major: 0,
            minor: 0,
            patch: 0,
            pre: vec![AlphaNumeric(String::from(trimmed))],
            build: Vec::new(),
        },
    };

    version
}

pub async fn fetch_github_tags(
    client: &Client,
    name: impl AsRef<str>,
    user: impl AsRef<str>,
    repo: impl AsRef<str>,
) -> anyhow::Result<BTreeMap<Version, Tag>> {
    let name = name.as_ref();
    let user = user.as_ref();
    let repo = repo.as_ref();
    let url = format!("https://api.github.com/repos/{}/{}/tags", &user, &repo);
    let bytes = client.get(&name, "tags.json", url.as_str()).await?;
    let tags: Vec<Tag> = serde_json::from_slice(&bytes)?;

    Ok(tags
        .into_iter()
        .map(|tag| (parse_version(tag.name.as_str()), tag))
        .collect())
}

#[derive(Debug, Deserialize)]
pub struct Ref {
    #[serde(rename = "ref")]
    pub reference: PathBuf,
    pub url: String,
    pub object: Object,
}

#[derive(Debug, Deserialize)]
pub struct Object {
    pub sha: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub url: Url,
}

pub async fn fetch_github_refs(
    client: &Client,
    name: impl AsRef<str>,
    user: impl AsRef<str>,
    repo: impl AsRef<str>,
) -> anyhow::Result<HashMap<PathBuf, Ref>> {
    let name = name.as_ref();
    let user = user.as_ref();
    let repo = repo.as_ref();
    let url = format!(
        "https://api.github.com/repos/{}/{}/git/refs/tags",
        &user, &repo
    );

    let bytes = client.get(&name, "refs.json", url.as_str()).await?;
    let refs: Vec<Ref> = serde_json::from_slice(&bytes)?;

    Ok(refs
        .into_iter()
        .map(|reference| (reference.reference.clone(), reference))
        .collect())
}
