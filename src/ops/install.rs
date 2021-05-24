use crate::atom::Atom;
use crate::config::Config;
use crate::external::autotools::Autotools;
use crate::external::cmake;
use crate::external::meson;
use crate::external::tar;
use crate::package::{Entry, Graph, PackageId};
use crate::shell::{Colour, Line, Text};
use crate::source::{github, gitlab, Source};
use crossterm::style::Colorize;
use semver::{Version, VersionReq};
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::fs;

pub mod build {
    use crate::config::Config;
    use crate::package::Entry;
    use semver::Version;
    use std::ops::Deref;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    crate struct Ref {
        config: Config,
        build_dir: PathBuf,
        install_dir: PathBuf,
    }

    pub struct Build(Arc<Ref>);

    impl Build {
        pub fn new(config: &Config, entry: &Entry, version: &Version) -> Self {
            let group_id = &entry.node().group_id;
            let package_id = &entry.node().package_id;

            let build_dir = config.build_with(|mut path| {
                path.push("x86_64-unknown-linux-gnu");
                path.push(group_id.as_str());
                path.push(package_id.as_str());
                path.push(version.to_string());
                path
            });

            let install_dir = config.target_with("x86_64-unknown-linux-gnu", |mut path| {
                path.push(group_id.as_str());
                path.push(package_id.as_str());
                path.push(version.to_string());
                path
            });

            Self(Arc::new(Ref {
                config: config.clone(),
                build_dir,
                install_dir,
            }))
        }

        pub fn config(&self) -> &Config {
            &self.0.config
        }

        pub fn build_dir(&self) -> &Path {
            self.0.build_dir.as_path()
        }

        pub fn install_dir(&self) -> &Path {
            self.0.install_dir.as_path()
        }
    }

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
                let build = build::Build::new(config, &entry, &source.0);
                let build_dir = build.build_dir();

                let buffer = unsafe {
                    let result = ufmt::uformat!(
                        "{}/{} v{}\n",
                        group_id.as_str().blue().to_string(),
                        package_id.as_str().green().to_string(),
                        &source.0.to_string(),
                    );

                    result.unwrap_unchecked()
                };

                Text::new(buffer).render(config.shell()).await?;

                if build.install_dir().exists() {
                    Line::new(" ->", Colour::None)
                        .append("installed", Colour::Green)
                        .newline()
                        .render(config.shell())
                        .await?;

                    continue;
                }

                let _ = fs::remove_dir_all(&build_dir).await;
                let entries = tar::extract(config, &source.1, &build_dir).await?;

                if let Some(root) = entries.iter().next() {
                    let root = build_dir.join(&root);

                    if root.join("CMakeLists.txt").exists() {
                        let mut cmake = cmake::cmake(&root);

                        cmake.prefix(build.install_dir());
                        cmake.execute(&build).await?;
                    } else if root.join("meson.build").exists() {
                        let mut meson = meson::meson(&root);

                        meson.prefix(build.install_dir());
                        meson.execute(&build).await?;
                    } else if root.join("configure").exists() {
                        let mut autotools = Autotools::new(&root);

                        autotools.prefix(build.install_dir());
                        autotools.execute(&build).await?;
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
