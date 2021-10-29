#![feature(format_args_nl)]

use command_extra::Command;
use futures_util::future;
use mix_id::RepositoryId;
use mix_shell::{header, AsyncWrite};
use path::PathBuf;
use std::collections::BTreeSet;
use url::Url;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct Config {
    pub prefix: PathBuf,
    pub repositories: BTreeSet<RepositoryId>,
}

pub async fn sync(global: mix_config::Config, config: Config) -> Result<()> {
    let repositories: Vec<_> = if config.repositories.is_empty() {
        global.repositories().iter().collect()
    } else {
        global
            .repositories()
            .iter()
            .filter(|(id, _url)| config.repositories.contains(id.as_str()))
            .collect()
    };

    let futures = repositories
        .iter()
        .map(|(id, url)| sync_repo(global.clone(), id, url))
        .collect::<Vec<_>>();

    let _results = future::join_all(futures).await;

    Ok(())
}

async fn sync_repo(config: mix_config::Config, id: &RepositoryId, url: &Url) -> Result<()> {
    header!(config.shell(), "sync {}", &id)?;

    let _ = config.shell().flush().await;
    let mut git = Command::new("git");
    let path = config.repos_prefix().join(id.as_str());

    if path.exists_async().await {
        git.current_dir(path).arg("pull");

        let mut child = git.spawn().await?;

        child.wait().await?;
    } else {
        let _ = path.create_dir_all_async().await;

        git.current_dir(path)
            .arg("clone")
            .arg("--depth=1")
            .arg(format!("{}", url))
            .arg(".");

        let mut child = git.spawn().await?;

        child.wait().await?;
    }

    Ok(())
}
