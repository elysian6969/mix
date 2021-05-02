use crate::shell::Text;
use crate::Config;
use std::path::Path;
use tokio::process::Command;

pub async fn fetch(config: &Config, path: impl AsRef<Path>) -> crate::Result<()> {
    Text::new(" -> fetch\n").render(config.shell()).await?;

    Command::new("git")
        .arg("fetch")
        .current_dir(path)
        .spawn()?
        .wait()
        .await?;

    Ok(())
}

pub async fn merge(config: &Config, path: impl AsRef<Path>) -> crate::Result<()> {
    Text::new(" -> merge\n").render(config.shell()).await?;

    Command::new("git")
        .arg("merge")
        .current_dir(path)
        .spawn()?
        .wait()
        .await?;

    Ok(())
}

pub async fn clone(config: &Config, path: &impl AsRef<Path>, url: &str) -> crate::Result<()> {
    Text::new(" -> clone\n").render(config.shell()).await?;

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
