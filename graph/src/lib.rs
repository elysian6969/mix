use crate::util;
use bimap::BiHashMap;
use futures::stream::{StreamExt, TryStreamExt};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tokio::fs::DirEntry;
use tokio::{fs, io};
use ufmt::derive::uDebug;
use vfs::VfsPath;

pub use self::display::Display;
pub use self::metadata::Metadata;
pub use self::node::Node;
pub use self::symbols::Symbols;

mod display;
mod metadata;
mod node;
mod opaque;
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
    let slice = entry.path().join("package.yml").read().await?;
    let metadata: Metadata = serde_yaml::from_slice(&slice)?;
    let package = Node::new(group_id, package_id.clone(), metadata);

    Ok((package_id, package))
}

#[derive(Debug)]
pub struct Graph {
    /// Group to package associations
    ///
    ///   group <-> package
    ///
    g2p: BiHashMap<opaque::Group, opaque::Package>,

    /// Package to package associations
    ///
    ///  package <-> package
    ///
    p2p: BiHashMap<opaque::Relation, opaque::Relation>,

    /// Package data
    p2d: HashMap<opaque::Package, Data>,
}

impl Graph {
    pub fn new() -> Graph {
        Self {
            g2p: BiHashMap::new(),
            p2p: BiHashMap::new(),
            p2d: HashMap::new(),
        }
    }

    pub async fn open(
        repositories: impl Iterator<Item = impl AsRef<VfsPath>>,
    ) -> crate::Result<Graph> {
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
            let entries = path.join("packages").read_dir().await?;
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

    pub fn get(&self, package_id: &PackageId) -> Option<Entry<'_>> {
        self.nodes.get(package_id).and_then(|node| {
            self.relationships
                .get(package_id)
                .map(|relationships| Entry {
                    node,
                    relationships,
                })
        })
    }

    pub fn order<'graph>(&'graph self, package_id: &'graph PackageId) -> Order<'graph> {
        let mut visited_packages: HashSet<&'graph PackageId> = HashSet::new();
        let mut order = Vec::new();

        depends_resolve(&self, &package_id, &mut visited_packages, &mut order);

        let mut visited_order: HashSet<&'graph PackageId> = HashSet::new();

        order
            .drain_filter(|package_id| visited_order.insert(package_id))
            .for_each(drop);

        Order {
            graph: self,
            visited: visited_packages,
            order,
        }
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

pub struct Entry<'graph> {
    pub node: &'graph Node,
    pub relationships: &'graph BTreeMap<PackageId, Relationship>,
}

impl<'graph> Entry<'graph> {
    pub fn node(&'graph self) -> &'graph Node {
        self.node
    }

    pub fn relationships(&'graph self) -> &'graph BTreeMap<PackageId, Relationship> {
        self.relationships
    }
}

pub struct Order<'graph> {
    graph: &'graph Graph,
    visited: HashSet<&'graph PackageId>,
    order: Vec<&'graph PackageId>,
}

impl<'graph> Order<'graph> {
    pub fn get(&self, package_id: &PackageId) -> Option<Entry<'_>> {
        self.visited
            .get(package_id)
            .and_then(|_| self.graph.get(package_id))
    }

    pub fn iter(&self) -> OrderIter {
        OrderIter {
            order: self,
            iter: self.order.iter(),
        }
    }
}

pub struct OrderIter<'graph> {
    order: &'graph Order<'graph>,
    iter: std::slice::Iter<'graph, &'graph PackageId>,
}

impl<'graph> Iterator for OrderIter<'graph> {
    type Item = Entry<'graph>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(package_id) => self.order.get(package_id),
            None => None,
        }
    }
}

fn depends_resolve<'graph>(
    graph: &'graph Graph,
    package_id: &'graph PackageId,
    visited_packages: &mut HashSet<&'graph PackageId>,
    dependency_order: &mut Vec<&'graph PackageId>,
) {
    if let Some(entry) = graph.get(package_id) {
        let visited = !visited_packages.insert(package_id);

        if visited {
            dependency_order.push(package_id);

            return;
        }

        for (package_id, _relationship) in entry.relationships.iter().rev() {
            depends_resolve(graph, package_id, visited_packages, dependency_order);
        }

        dependency_order.push(package_id);
    }
}
