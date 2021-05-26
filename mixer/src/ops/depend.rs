use crate::atom::Atom;
use crate::config::Config;
use crate::package::{Graph, PackageId, Symbols};
use crate::shell::Text;
use std::collections::HashSet;

pub async fn depend(config: &Config, atoms: HashSet<Atom>) -> crate::Result<()> {
    let repositories = config.repositories().keys();
    let graph = Graph::open(repositories).await?;

    for atom in atoms.iter() {
        let buffer = ufmt::uformat!("{:?}\n", &atoms).expect("infallible");

        Text::new(buffer).render(config.shell()).await?;

        let package_id = PackageId::new(&atom.package);

        Text::new(graph.display_tree(&package_id, &Symbols::utf8()))
            .render(config.shell())
            .await?;
    }

    Ok(())
}