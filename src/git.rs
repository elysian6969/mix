use crate::shell::{Shell, Text};
use std::path::Path;
use tokio::process::Command;

pub async fn fetch(shell: &Shell, path: impl AsRef<Path>) -> crate::Result<()> {
    Text::new(" -> fetch\n").render(shell).await?;

    Command::new("git")
        .arg("fetch")
        .current_dir(path)
        .spawn()?
        .wait()
        .await?;

    Ok(())
}

pub async fn merge(shell: &Shell, path: impl AsRef<Path>) -> crate::Result<()> {
    Text::new(" -> merge\n").render(shell).await?;

    Command::new("git")
        .arg("merge")
        .current_dir(path)
        .spawn()?
        .wait()
        .await?;

    Ok(())
}

pub async fn clone(shell: &Shell, path: &impl AsRef<Path>, url: &str) -> crate::Result<()> {
    Text::new(" -> clone\n").render(shell).await?;

    Command::new("git")
        .arg("clone")
        .arg(url)
        .arg(".")
        .current_dir(path)
        .spawn()?
        .wait()
        .await?;

    Ok(())
}
