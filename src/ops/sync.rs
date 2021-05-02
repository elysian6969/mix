use crate::config::Config;
use crate::git;
use crate::{PREFIX, REPOSITORY};
use std::path::Path;

pub async fn sync(config: &Config) -> crate::Result<()> {
    let repository = Path::new(PREFIX).join("repository");

    if repository.exists() {
        git::fetch(&config, &repository).await?;
        git::merge(&config, &repository).await?;
    } else {
        git::clone(&config, &repository, REPOSITORY).await?;
    }

    Ok(())
}
