use {
    super::client::Client,
    serde::Deserialize,
    hashbrown::HashMap,
    std::path::PathBuf,
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

pub async fn fetch_github_tags(
    client: &Client,
    name: impl AsRef<str>,
    user: impl AsRef<str>,
    repo: impl AsRef<str>,
) -> anyhow::Result<HashMap<String, Tag>> {
    let name = name.as_ref();
    let user = user.as_ref();
    let repo = repo.as_ref();
    let url = format!("https://api.github.com/repos/{}/{}/tags", &user, &repo);
    let bytes = client.get(&name, url.as_str()).await?;
    
    dbg!(&url);

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

pub async fn fetch_github_refs(
    client: &Client,
    name: impl AsRef<str>,
    user: impl AsRef<str>,
    repo: impl AsRef<str>,
) -> anyhow::Result<HashMap<String, Ref>> {
    let name = name.as_ref();
    let user = user.as_ref();
    let repo = repo.as_ref();
    let url = format!("https://api.github.com/repos/{}/{}/git/refs/tags", &user, &repo);
    let bytes = client.get(&name, url.as_str()).await?;
    
    dbg!(&url);

    Ok(HashMap::new())

    /*Ok(tags
        .into_iter()
        .flat_map(|tag| {
            Some((
                tag.reference.file_name()?.to_string_lossy().to_string(),
                tag,
            ))
        })
        .collect())*/
}
