use crate::atom::Atom;
use crate::config::Config;
use crate::package::{Graph, PackageId, Symbols};
use crate::shell::Text;
use std::collections::HashSet;

pub async fn depend(config: &Config, atoms: HashSet<Atom>) -> crate::Result<()> {
    let graph = Graph::open(config.prefix().join("repository/core")).await?;

    for atom in atoms {
        let package_id = PackageId::new(atom.name);

        Text::new(graph.display_tree(&package_id, &Symbols::utf8()))
            .render(config.shell())
            .await?;

        let order = graph.dependency_order(&package_id);

        dbg!(order);
    }

    Ok(())
}
