use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Tag {
    pub commit: Option<Commit>,
    pub release: Option<Release>,
    pub name: Option<String>,
    pub target: Option<String>,
    pub message: Option<String>,
    pub protected: Option<bool>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct Release {
    pub tag_name: Option<String>,
    pub description: Option<String>,
}
