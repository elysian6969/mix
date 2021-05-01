use crate::atom::Atom;
use crate::git;
use crate::package::{Graph, PackageId, UTF8_SYMBOLS};
use crate::shell::{Shell, Text};
use crate::{PREFIX, REPOSITORY};
use std::collections::HashSet;
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
