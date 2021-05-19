use crate::atom::Atom;
use crate::config::Config;
use crate::external::autotools::Autotools;
use crate::external::tar;
use crate::package::{Entry, Graph, PackageId};
use crate::shell::Text;
use crate::source::{github, gitlab, Source};
use crossterm::style::Colorize;
use semver::{Version, VersionReq};
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::fs;

pub mod build {
    use crate::config::Config;
    use std::ops::Deref;
    use std::path::PathBuf;
    use std::sync::Arc;

    crate struct Ref {
        config: Config,
        build_dir: PathBuf,
        install_dir: PathBuf,
    }

    pub struct Build(Arc<Ref>);

    impl Deref for Build {
        type Target = Config;

        fn deref(&self) -> &Self::Target {
            &self.0.config
        }
    }
}

pub async fn install(config: &Config, atoms: HashSet<Atom>) -> crate::Result<()> {
    let repositories = config.repositories().keys();
    let graph = Graph::open(repositories).await?;
    let requirement = VersionReq::parse("*")?;

    for atom in atoms {
        let package_id = PackageId::new(&atom.package);
        let order = graph.order(&package_id);

        for entry in order.iter() {
            let group_id = &entry.node().group_id;
            let package_id = &entry.node().package_id;
            let (sources, errors) = download_sources(config, &entry, &requirement).await?;

            for source in sources {
                let build_dir = config.build_with(|mut path| {
                    path.push(group_id.as_str());
                    path.push(package_id.as_str());
                    path.push(source.0.to_string());
                    path
                });

                let buffer = ufmt::uformat!(
                    "{}/{} {:?}\n",
                    group_id.as_str().blue().to_string(),
                    package_id.as_str().green().to_string(),
                    build_dir
                )
                .expect("infallible");

                Text::new(buffer).render(config.shell()).await?;

                let _ = fs::remove_dir_all(&build_dir).await;
                let entries = tar::extract(config, &source.1, &build_dir).await?;

                if let Some(root) = entries.iter().next() {
                    let root = build_dir.join(&root);

                    if root.join("configure").exists() {
                        let mut autotools = Autotools::new(&root);

                        autotools.execute(config).await?;
                    }
                }

                // implement tracking to reduce i/o
                // {build}/{group}/{package}/{version}
                let _ = fs::remove_dir_all(&build_dir).await;

                for ancestor in build_dir.ancestors().take(2) {
                    let _ = fs::remove_dir(&ancestor).await;
                }
            }
        }
    }

    Ok(())
}

async fn download_sources(
    config: &Config,
    entry: &Entry<'_>,
    requirement: &VersionReq,
) -> crate::Result<(Vec<(Version, PathBuf)>, Vec<crate::Error>)> {
    let mut sources = vec![];
    let mut errors = vec![];

    for source in entry.node().metadata.source.iter() {
        match &source {
            Source::Github { user, repo } => {
                match download_github(config, entry, user, repo, requirement).await {
                    Ok(entry) => sources.push(entry),
                    Err(error) => errors.push(error),
                }
            }
            Source::Gitlab { user, repo } => {
                match download_gitlab(config, entry, user, repo, requirement).await {
                    Ok(entry) => sources.push(entry),
                    Err(error) => errors.push(error),
                }
            }
            _ => {}
        }
    }

    Ok((sources, errors))
}

async fn download_github(
    config: &Config,
    entry: &Entry<'_>,
    user: &str,
    repo: &str,
    requirement: &VersionReq,
) -> crate::Result<(Version, PathBuf)> {
    let repo = github::Repo::new(user, repo);
    let tags = repo.tags(config).await?;
    let matches = tags.matches(requirement);

    if let Some(tag) = matches.newest() {
        /*let group_id = &entry.node().group_id;
        let package_id = &entry.node().package_id;
        let buffer = ufmt::uformat!(
            "{}/{} @ v{} -> {}\n",
            group_id,
            package_id,
            tag.version().to_string(),
            tag.url().as_str()
        )
        .expect("infallible");

        Text::new(buffer).render(config.shell()).await?;*/

        tag.download(config).await?;

        Ok((tag.version().clone(), tag.path().to_path_buf()))
    } else {
        Err("no source".into())
    }
}

async fn download_gitlab(
    config: &Config,
    entry: &Entry<'_>,
    user: &str,
    repo: &str,
    requirement: &VersionReq,
) -> crate::Result<(Version, PathBuf)> {
    let repo = gitlab::Repo::new(gitlab::gitlab_url(), user, repo);
    let tags = repo.tags(config).await?;
    let matches = tags.matches(requirement);

    if let Some(tag) = matches.newest() {
        /*let group_id = &entry.node().group_id;
        let package_id = &entry.node().package_id;
        let buffer = ufmt::uformat!(
            "{}/{} @ v{} -> {}\n",
            group_id,
            package_id,
            tag.version().to_string(),
            tag.url().as_str()
        )
        .expect("infallible");

        Text::new(buffer).render(config.shell()).await?;*/

        tag.download(config).await?;

        Ok((tag.version().clone(), tag.path().to_path_buf()))
    } else {
        Err("no source".into())
    }
}
