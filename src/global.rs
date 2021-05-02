use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub repositories: BTreeMap<PathBuf, Url>,
}

impl Metadata {
    pub async fn open(path: impl AsRef<Path>) -> crate::Result<Self> {
        let slice = &fs::read(path).await?;
        let metadata = serde_yaml::from_slice(&slice)?;

        Ok(metadata)
    }
}
