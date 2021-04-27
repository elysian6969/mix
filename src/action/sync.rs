use crate::args::Sync;
use crate::git;
use crate::{PREFIX, REPOSITORY};
use std::path::Path;

pub async fn sync(_sync: Sync) -> anyhow::Result<()> {
    let prefix = Path::new(PREFIX);
    let repository_path = prefix.join("repository");

    if repository_path.exists() {
        let repository = git2::Repository::open(&repository_path)?;

        git::remote_fetch(&repository).await?;
    } else {
        git::clone(REPOSITORY, &repository_path).await?;
    }

    Ok(())
}
