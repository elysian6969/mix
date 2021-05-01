use crate::git;
use crate::shell::Shell;
use crate::{PREFIX, REPOSITORY};
use std::path::Path;

pub async fn sync(shell: &Shell) -> crate::Result<()> {
    let repository = Path::new(PREFIX).join("repository");

    if repository.exists() {
        git::fetch(&shell, &repository).await?;
        git::merge(&shell, &repository).await?;
    } else {
        git::clone(&shell, &repository, REPOSITORY).await?;
    }

    Ok(())
}
