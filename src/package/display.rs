use super::{Graph, Node, PackageId, Symbols};
use crossterm::style::Colorize;
use std::collections::HashSet;
use std::fmt;

pub struct Display<'graph, 'symbols> {
    pub graph: &'graph Graph,
    pub root: &'graph PackageId,
    pub symbols: &'symbols Symbols,
}

impl<'graph, 'symbols> fmt::Display for Display<'graph, 'symbols> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // set of visited packages otherwise circular
        // dependencies end in stack overflow
        let mut visited_packages = HashSet::new();
        // maintain where branches are
        let mut levels = Vec::new();

        print_tree(
            f,
            self.graph,
            self.root,
            self.symbols,
            &mut visited_packages,
            &mut levels,
        )?;

        Ok(())
    }
}

/// print a package and it's details
fn print_package<'graph>(
    f: &mut fmt::Formatter,
    _graph: &'graph Graph,
    node: &'graph Node,
    visited_packages: &mut HashSet<&'graph PackageId>,
) -> Result<bool, fmt::Error> {
    // insert returns false when they key already exists
    let visited = !visited_packages.insert(&node.package_id);
    let star = if visited { " (*)" } else { "" };

    writeln!(
        f,
        "{}/{}{star}",
        node.group_id.as_str().green(),
        node.package_id.as_str().green()
    )?;

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
fn print_tree<'graph>(
    f: &mut fmt::Formatter,
    graph: &'graph Graph,
    package_id: &'graph PackageId,
    symbols: &Symbols,
    visited_packages: &mut HashSet<&'graph PackageId>,
    levels: &mut Vec<bool>,
) -> fmt::Result {
    if let Some((node, relationships)) = graph.get(package_id) {
        print_branches(f, levels, symbols)?;

        let visited = print_package(f, graph, node, visited_packages)?;

        // don't recursively enumerate dependencies
        // zero dependencies means we needn't print anything
        if visited || relationships.is_empty() {
            return Ok(());
        }

        for (index, (package_id, _relationships)) in relationships.iter().enumerate() {
            // the last package is the tail
            // inbetween is either a tee or a down
            let is_last = index == relationships.len().saturating_sub(1);

            levels.push(!is_last);
            print_tree(f, graph, package_id, symbols, visited_packages, levels)?;
            levels.pop();
        }
    } else {
        // fixme
        println!("error: missing {package_id}");
    }

    Ok(())
}
