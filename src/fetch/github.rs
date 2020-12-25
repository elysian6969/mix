use {
    reqwest::{header, Client},
    serde::Deserialize,
    std::{collections::HashMap, path::PathBuf},
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

pub async fn fetch_github_tags(user: &str, repo: &str) -> anyhow::Result<HashMap<String, Tag>> {
    let url = format!("https://api.github.com/repos/{}/{}/tags", &user, &repo);

    let text: String = Client::new()
        .get(&url)
        .header(
            header::USER_AGENT,
            concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
        )
        .send()
        .await?
        .text()
        .await?;

    println!("{}", &text);

    Ok(HashMap::new())

    /*
    Ok(tags
        .into_iter()
        .map(|tag| (tag.name.clone(), tag))
        .collect())*/
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

pub async fn fetch_github_refs(user: &str, repo: &str) -> anyhow::Result<HashMap<String, Ref>> {
    let tags: Vec<Ref> = reqwest::get(&format!(
        "https://api.github.com/{}/{}/git/refs/tags",
        &user, &repo
    ))
    .await?
    .json()
    .await?;

    Ok(tags
        .into_iter()
        .flat_map(|tag| {
            Some((
                tag.reference.file_name()?.to_string_lossy().to_string(),
                tag,
            ))
        })
        .collect())
}
