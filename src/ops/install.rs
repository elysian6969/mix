use crate::atom::Atom;
use crate::config::Config;
use crate::package::{Graph, PackageId};
use crate::shell::Text;
use std::collections::HashSet;

pub async fn install(config: &Config, atoms: HashSet<Atom>) -> crate::Result<()> {
    let graph = Graph::open(config.prefix().join("repository/core")).await?;

    for atom in atoms {
        let package_id = PackageId::new(atom.name);
        let order = graph.dependency_order(&package_id);

        for package_id in order {
            let (package, _relationships) = graph.get(&package_id).expect("should always be some");

            for source in package.metadata.source.iter() {
                let scheme = source.url.scheme();
                let path = source.url.path();
                let url = match scheme {
                    "github" => format!("https://github.com/{path}"),
                    _ => return Ok(()),
                };

                Text::new(format!("{url}\n")).render(config.shell()).await?;
            }
        }
    }

    Ok(())
}
