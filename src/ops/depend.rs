use crate::atom::Atom;
use crate::config::Config;
use crate::package::{Graph, PackageId, Symbols};
use crate::shell::Text;
use crate::PREFIX;
use std::collections::HashSet;
use std::path::Path;

pub async fn depend(config: &Config, atoms: HashSet<Atom>) -> crate::Result<()> {
    let core = Path::new(PREFIX).join("repository/core");
    let graph = Graph::open(&core).await?;

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
