use super::{Graph, Node, PackageId, Symbols};
use crossterm::style::Stylize;
use std::collections::HashSet;

pub struct Display<'graph, 'symbols> {
    pub graph: &'graph Graph,
    pub root: &'graph PackageId,
    pub symbols: &'symbols Symbols,
}

impl<'graph, 'symbols> ufmt::uDisplay for Display<'graph, 'symbols> {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        // set of visited packages otherwise circular
        // dependencies end in stack overflow
        let mut visited_packages = HashSet::new();
        // maintain where branches are
        let mut levels = Vec::new();

        print_tree::<W>(
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
fn print_package<'graph, W>(
    f: &mut ufmt::Formatter<'_, W>,
    _graph: &'graph Graph,
    node: &'graph Node,
    visited_packages: &mut HashSet<&'graph PackageId>,
) -> Result<bool, W::Error>
where
    W: ufmt::uWrite + ?Sized,
{
    // insert returns false when they key already exists
    let visited = !visited_packages.insert(&node.package_id);
    let star = if visited { " (*)" } else { "" };

    ufmt::uwriteln!(
        f,
        "{}/{}{}",
        node.group_id.as_str().blue().to_string(),
        node.package_id.as_str().green().to_string(),
        star,
    )?;

    Ok(visited)
}

/// print the tree's branches
fn print_branches<W>(
    f: &mut ufmt::Formatter<'_, W>,
    levels: &mut Vec<bool>,
    symbols: &Symbols,
) -> Result<(), W::Error>
where
    W: ufmt::uWrite + ?Sized,
{
    if let Some((last, rest)) = levels.split_last() {
        for branch in rest {
            let character = if *branch { symbols.down } else { " " };
            ufmt::uwrite!(f, "{}   ", character)?;
        }

        let character = if *last { symbols.tee } else { symbols.ell };
        ufmt::uwrite!(f, "{}{}{} ", character, symbols.right, symbols.right)?;
    }

    Ok(())
}

/// print a dependency tree
fn print_tree<'graph, W>(
    f: &mut ufmt::Formatter<'_, W>,
    graph: &'graph Graph,
    package_id: &'graph PackageId,
    symbols: &Symbols,
    visited_packages: &mut HashSet<&'graph PackageId>,
    levels: &mut Vec<bool>,
) -> Result<(), W::Error>
where
    W: ufmt::uWrite + ?Sized,
{
    if let Some(entry) = graph.get(package_id) {
        print_branches(f, levels, symbols)?;

        let visited = print_package(f, graph, entry.node, visited_packages)?;

        // don't recursively enumerate dependencies
        // zero dependencies means we needn't print anything
        if visited || entry.relationships.is_empty() {
            return Ok(());
        }

        for (index, (package_id, _relationships)) in entry.relationships.iter().enumerate() {
            // the last package is the tail
            // inbetween is either a tee or a down
            let is_last = index == entry.relationships.len().saturating_sub(1);

            levels.push(!is_last);
            print_tree(f, graph, package_id, symbols, visited_packages, levels)?;
            levels.pop();
        }
    } else {
        // TODO: sanitise the tree lmao?
        let buffer = ufmt::uformat!("{}", package_id).expect("infallible");

        println!("error: missing {buffer}");
    }

    Ok(())
}
