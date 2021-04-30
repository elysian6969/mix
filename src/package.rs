use crate::source::Source;
use crate::util;
use futures::stream::{StreamExt, TryStreamExt};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;
use tokio::fs::DirEntry;
use tokio::{fs, io};

#[derive(Debug, Deserialize)]
pub struct Metadata {
    #[serde(default = "BTreeSet::new")]
    depends: BTreeSet<String>,
    source: BTreeSet<Source>,
}

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
    pub nodes: BTreeMap<PackageId, Package>,
    /// relationships between packages
    pub relations: BTreeMap<PackageId, BTreeMap<PackageId, Relation>>,
}

impl Graph {
    pub async fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let packages: BTreeMap<_, _> = util::read_dir(path.as_ref().join("packages"))
            .await?
            .then(map_entry)
            .try_collect()
            .await?;

        let mut graph = Graph {
            nodes: BTreeMap::new(),
            relations: BTreeMap::new(),
        };

        for (id, package) in packages.into_iter() {
            graph.relations.insert(id.clone(), BTreeMap::new());

            for depend in package.metadata.depends.iter() {
                graph
                    .relations
                    .get_mut(&id)
                    .expect("already inserted")
                    .insert(PackageId::new(depend), Relation::Direct);
            }

            graph.nodes.insert(id, package);
        }

        Ok(graph)
    }

    pub fn get(&self, id: &PackageId) -> Option<(&Package, &BTreeMap<PackageId, Relation>)> {
        self.nodes
            .get(id)
            .and_then(|package| self.relations.get(id).map(|relations| (package, relations)))
    }

    pub fn display<'graph, 'symbols>(
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

pub struct Display<'graph, 'symbols> {
    graph: &'graph Graph,
    root: &'graph PackageId,
    symbols: &'symbols Symbols,
}

impl<'graph, 'symbols> fmt::Display for Display<'graph, 'symbols> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        print(f, &self.graph, &self.root, &self.symbols)
    }
}

use std::collections::HashSet;

pub struct Symbols {
    down: &'static str,
    tee: &'static str,
    ell: &'static str,
    right: &'static str,
}

#[allow(dead_code)]
pub static UTF8_SYMBOLS: Symbols = Symbols {
    down: "│",
    tee: "├",
    ell: "└",
    right: "─",
};

#[allow(dead_code)]
pub static ASCII_SYMBOLS: Symbols = Symbols {
    down: "|",
    tee: "|",
    ell: "`",
    right: "-",
};

/// print a dependency tree starting from a package
fn print<'package>(
    f: &mut fmt::Formatter,
    graph: &'package Graph,
    package_id: &'package PackageId,
    symbols: &Symbols,
) -> fmt::Result {
    // set of visited packages otherwise circular
    // dependencies end in stack overflow
    let mut visited_packages = HashSet::new();
    // maintain where branches are
    let mut levels = Vec::new();

    print_tree(
        f,
        graph,
        package_id,
        symbols,
        &mut visited_packages,
        &mut levels,
    )?;

    Ok(())
}

/// print a package and it's details
fn print_package<'package>(
    f: &mut fmt::Formatter,
    _graph: &'package Graph,
    package_id: &'package PackageId,
    visited_packages: &mut HashSet<&'package PackageId>,
) -> Result<bool, fmt::Error> {
    use crossterm::style::Colorize;

    // insert returns false when they key already exists
    let visited = !visited_packages.insert(package_id);
    let star = if visited { " (*)" } else { "" };

    writeln!(f, "{}{star}", package_id.to_string().green())?;

    Ok(visited)
}

/// print the tree's branches
fn print_branches(
    f: &mut fmt::Formatter,
    levels: &mut Vec<bool>,
    symbols: &Symbols,
) -> fmt::Result {
    if let Some((last, rest)) = levels.split_last() {
        for branch in rest {
            let character = if *branch { symbols.down } else { " " };
            write!(f, "{}   ", character)?;
        }

        let character = if *last { symbols.tee } else { symbols.ell };
        write!(f, "{0}{1}{1} ", character, symbols.right)?;
    }

    Ok(())
}

/// print a dependency tree
fn print_tree<'package>(
    f: &mut fmt::Formatter,
    graph: &'package Graph,
    package_id: &'package PackageId,
    symbols: &Symbols,
    visited_packages: &mut HashSet<&'package PackageId>,
    levels: &mut Vec<bool>,
) -> fmt::Result {
    if let Some((_package, relationships)) = graph.get(package_id) {
        print_branches(f, levels, symbols)?;

        let visited = print_package(f, graph, package_id, visited_packages)?;

        // don't recursively enumerate dependencies
        if visited {
            return Ok(());
        }

        // zero dependencies means we needn't print anything
        if relationships.is_empty() {
            return Ok(());
        }

        for (index, (package_id, _relationship)) in relationships.iter().enumerate() {
            // the last package is the tail
            // inbetween is either a tee or a down
            let is_last = index == relationships.len() - 1;

            levels.push(!is_last);
            print_tree(f, graph, package_id, symbols, visited_packages, levels)?;
            levels.pop();
        }
    } else {
        println!("error: missing {package_id}");
    }

    Ok(())
}
