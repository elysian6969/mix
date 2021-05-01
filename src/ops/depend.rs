use crate::atom::Atom;
use crate::package::{Graph, PackageId, UTF8_SYMBOLS};
use crate::shell::{Shell, Text};
use crate::PREFIX;
use std::collections::HashSet;
use std::path::Path;

pub async fn depend(shell: &Shell, atoms: HashSet<Atom>) -> crate::Result<()> {
    let core = Path::new(PREFIX).join("repository/core");
    let graph = Graph::open(&core).await?;

    for atom in atoms {
        let package_id = PackageId::new(atom.name);

        Text::new(graph.display(&package_id, &UTF8_SYMBOLS))
            .render(&shell)
            .await?;
    }

    Ok(())
}
