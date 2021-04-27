use crate::source::Source;
use crate::util;
use futures::stream::{StreamExt, TryStreamExt};
use serde::Deserialize;
use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use tokio::fs::DirEntry;
use tokio::{fs, io};

#[derive(Debug, Deserialize)]
pub struct Metadata {
    #[serde(default = "BTreeSet::new")]
    depends: BTreeSet<String>,
    source: BTreeSet<Source>,
}

use std::fmt;

#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct PackageId {
    id: String,
}

impl PackageId {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl fmt::Debug for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.id, f)
    }
}

impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.id, f)
    }
}

#[derive(Debug)]
pub struct Package {
    pub package_id: PackageId,
    pub metadata: Metadata,
}

impl Package {
    pub fn new(package_id: PackageId, metadata: Metadata) -> Self {
        Package {
            package_id,
            metadata,
        }
    }
}

async fn map_entry(entry: io::Result<DirEntry>) -> anyhow::Result<(PackageId, Package)> {
    let entry = entry?;
    let file_name = entry
        .file_name()
        .into_string()
        .map_err(|_| anyhow::anyhow!("invalid utf-8"))?;

    let package_id = PackageId::new(file_name);
    let config = entry.path().join("package.yml");
    let slice = &fs::read(config).await?;
    let metadata: Metadata = serde_yaml::from_slice(&slice)?;
    let package = Package::new(package_id.clone(), metadata);

    Ok((package_id, package))
}

#[derive(Debug)]
pub enum Relation {
    Build,
    Direct,
    Runtime,
}

#[derive(Debug)]
pub struct Graph {
    /// packages themselves
    pub nodes: HashMap<PackageId, Package>,
    /// relationships between packages
    pub relations: HashMap<PackageId, HashMap<PackageId, Relation>>,
}

impl Graph {
    pub async fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let packages: HashMap<_, _> = util::read_dir(path.as_ref().join("packages"))
            .await?
            .then(map_entry)
            .try_collect()
            .await?;

        let mut graph = Graph {
            nodes: HashMap::new(),
            relations: HashMap::new(),
        };

        for (id, package) in packages.into_iter() {
            for depend in package.metadata.depends.iter() {
                graph
                    .relations
                    .entry(id.clone())
                    .or_insert_with(HashMap::new)
                    .insert(PackageId::new(depend), Relation::Direct);
            }

            graph.nodes.insert(id, package);
        }

        Ok(graph)
    }

    pub fn get(&self, id: &PackageId) -> Option<(&Package, &HashMap<PackageId, Relation>)> {
        self.nodes
            .get(id)
            .and_then(|package| self.relations.get(id).map(|relations| (package, relations)))
    }
}
