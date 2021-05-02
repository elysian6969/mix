use crate::atom::Atom;
use crate::config::Config;
use crate::package::{Graph, PackageId, Symbols};
use crate::shell::Text;
use std::collections::HashSet;

pub async fn depend(config: &Config, atoms: HashSet<Atom>) -> crate::Result<()> {
    let repositories = config.repositories().keys();
    let graph = Graph::open(repositories).await?;

    for atom in atoms.iter() {
        let package_id = PackageId::new(&atom.name);

        Text::new(graph.display_tree(&package_id, &Symbols::utf8()))
            .render(config.shell())
            .await?;
    }

    Ok(())
}
