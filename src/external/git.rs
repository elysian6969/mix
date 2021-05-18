use crate::shell::Text;
use crate::Config;
use std::path::Path;
use tokio::fs;
use tokio::process::Command;
use url::Url;

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

pub async fn clone(config: &Config, path: impl AsRef<Path>, url: &Url) -> crate::Result<()> {
    Text::new(" -> clone\n").render(config.shell()).await?;

    fs::create_dir_all(path.as_ref()).await?;

    Command::new("git")
        .arg("clone")
        .arg(url.as_str())
        .arg(".")
        .current_dir(path)
        .spawn()?
        .wait()
        .await?;

    Ok(())
}
