use crate::util;
use futures::stream::{StreamExt, TryStreamExt};
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::path::Path;
use tokio::fs::DirEntry;
use tokio::{fs, io};
use ufmt::derive::uDebug;

pub use self::display::Display;
pub use self::group_id::GroupId;
pub use self::metadata::Metadata;
pub use self::node::Node;
pub use self::package_id::PackageId;
pub use self::symbols::Symbols;

mod display;
mod group_id;
mod metadata;
mod node;
mod package_id;
mod symbols;

async fn map_entry(
    group_id: GroupId,
    entry: io::Result<DirEntry>,
) -> crate::Result<(PackageId, Node)> {
    let entry = entry?;
    let file_name = entry
        .file_name()
        .into_string()
        .map_err(|_| "invalid utf-8")?;

    let package_id = PackageId::new(file_name);
    let config = entry.path().join("package.yml");
    let slice = &fs::read(config).await?;
    let metadata: Metadata = serde_yaml::from_slice(&slice)?;
    let package = Node::new(group_id, package_id.clone(), metadata);

    Ok((package_id, package))
}

#[derive(uDebug)]
pub enum Relationship {
    Build,
    Direct,
    Runtime,
}

#[derive(uDebug)]
pub struct Graph {
    /// packages themselves
    pub nodes: BTreeMap<PackageId, Node>,
    /// relationships between packages
    pub relationships: BTreeMap<PackageId, BTreeMap<PackageId, Relationship>>,
}

impl Graph {
    pub async fn open(repositories: impl Iterator<Item = impl AsRef<Path>>) -> crate::Result<Self> {
        let mut graph = Graph {
            nodes: BTreeMap::new(),
            relationships: BTreeMap::new(),
        };

        for repository in repositories {
            let path = repository.as_ref();
            let name = path
                .file_name()
                .expect("file_name")
                .to_str()
                .expect("to_str");

            let group_id = GroupId::new(name);
            let entries = util::read_dir(path.join("packages")).await?;
            let packages: BTreeMap<_, _> = entries
                .then(|entry| map_entry(group_id.clone(), entry))
                .try_collect()
                .await?;

            for (id, node) in packages.into_iter() {
                graph.relationships.insert(id.clone(), BTreeMap::new());

                for depend in node.metadata.depends.iter() {
                    graph
                        .relationships
                        .get_mut(&id)
                        .expect("already inserted")
                        .insert(PackageId::new(depend), Relationship::Direct);
                }

                graph.nodes.insert(id, node);
            }
        }

        Ok(graph)
    }

    pub fn get(
        &self,
        package_id: &PackageId,
    ) -> Option<(&Node, &BTreeMap<PackageId, Relationship>)> {
        self.nodes.get(package_id).and_then(|node| {
            self.relationships
                .get(package_id)
                .map(|relationships| (node, relationships))
        })
    }

    pub fn dependency_order<'graph>(
        &'graph self,
        package_id: &'graph PackageId,
    ) -> Vec<&'graph PackageId> {
        let mut visited_packages = HashSet::new();
        let mut dependency_order = Vec::new();

        depends_resolve(
            &self,
            &package_id,
            &mut visited_packages,
            &mut dependency_order,
        );

        dependency_order
    }

    pub fn display_tree<'graph, 'symbols>(
        &'graph self,
        root: &'graph PackageId,
        symbols: &'symbols Symbols,
    ) -> Display<'graph, 'symbols> {
        Display {
            graph: &self,
            root,
            symbols,
        }
    }
}

fn depends_resolve<'graph>(
    graph: &'graph Graph,
    package_id: &'graph PackageId,
    visited_packages: &mut HashSet<&'graph PackageId>,
    dependency_order: &mut Vec<&'graph PackageId>,
) {
    if let Some((_node, relationships)) = graph.get(package_id) {
        let visited = !visited_packages.insert(package_id);

        if visited {
            dependency_order.push(package_id);

            return;
        }

        for (package_id, _relationship) in relationships.iter().rev() {
            depends_resolve(graph, package_id, visited_packages, dependency_order);
        }

        dependency_order.push(package_id);
    }
}
