use crate::atom::Atom;
use crate::config::Config;
use crate::package::{Graph, PackageId};
use crate::source::{github, gitlab, Source};
use std::collections::{BTreeSet, HashSet};

pub async fn install(config: &Config, atoms: HashSet<Atom>) -> crate::Result<()> {
    let repositories = config.repositories().keys();
    let graph = Graph::open(repositories).await?;

    for atom in atoms {
        let package_id = PackageId::new(&atom.package);
        let order = graph.order(&package_id);

        for entry in order.iter() {
            for source in entry.node().metadata.source.iter() {
                match &source {
                    Source::Github { user, repo } => {
                        let repo = github::Repo::new(user, repo);
                        let tags = repo.tags(config).await?;
                        let matches = tags.matches(&atom.version);
                        let package_id =
                            ufmt::uformat!("{}", &entry.node().package_id).expect("infallible");

                        if let Some(tag) = matches.oldest() {
                            println!("[{package_id}] oldest v{}\n - {}", tag.version(), tag.url());
                        }

                        if let Some(tag) = matches.newest() {
                            println!("[{package_id}] newest v{}\n - {}", tag.version(), tag.url());
                        }
                    }
                    Source::Gitlab { user, repo } => {
                        let repo = gitlab::Repo::new(gitlab::gitlab_url(), user, repo);
                        let tags = repo.tags(config).await?;
                        let matches = tags.matches(&atom.version);
                        let package_id =
                            ufmt::uformat!("{}", &entry.node().package_id).expect("infallible");

                        if let Some(tag) = matches.oldest() {
                            println!("[{package_id}] oldest v{}\n - {}", tag.version(), tag.url());
                        }

                        if let Some(tag) = matches.newest() {
                            println!("[{package_id}] newest v{}\n - {}", tag.version(), tag.url());
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
