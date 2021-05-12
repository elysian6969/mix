use crate::atom::Atom;
use crate::config::Config;
use crate::github::Repo;
use crate::package::{Graph, PackageId};
use crate::source::Source;
use std::collections::HashSet;

pub async fn install(config: &Config, atoms: HashSet<Atom>) -> crate::Result<()> {
    let repositories = config.repositories().keys();
    let graph = Graph::open(repositories).await?;

    for atom in atoms {
        let package_id = PackageId::new(atom.package);
        let order = graph.dependency_order(&package_id);

        for package_id in order {
            let (node, _relationships) = graph.get(&package_id).expect("should always be some");

            for source in node.metadata.source.iter() {
                match source {
                    Source::Github { user, repository } => {
                        let tags = Repo::new(user, repository).tags(config).await?;

                        println!("{tags:?}");
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
