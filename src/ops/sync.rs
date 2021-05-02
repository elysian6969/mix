use crate::config::Config;
use crate::git;

pub async fn sync(config: &Config) -> crate::Result<()> {
    for (path, url) in config.repositories().iter() {
        if path.exists() {
            git::fetch(&config, &path).await?;
            git::merge(&config, &path).await?;
        } else {
            git::clone(&config, &path, url).await?;
        }
    }

    Ok(())
}
