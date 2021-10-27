use serde::Deserialize;

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
