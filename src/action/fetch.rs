use crate::args::Fetch;
use crate::package::{Graph, PackageId};
use crate::PREFIX;
use std::collections::HashSet;
use std::path::Path;

struct Symbols {
    down: &'static str,
    tee: &'static str,
    ell: &'static str,
    right: &'static str,
}

static UTF8_SYMBOLS: Symbols = Symbols {
    down: "│",
    tee: "├",
    ell: "└",
    right: "─",
};

#[allow(dead_code)]
static ASCII_SYMBOLS: Symbols = Symbols {
    down: "|",
    tee: "|",
    ell: "`",
    right: "-",
};

/// print a dependency tree starting from a package
fn print<'package>(graph: &'package Graph, package_id: &'package PackageId, symbols: &Symbols) {
    // set of visited packages otherwise circular
    // dependencies end in stack overflow
    let mut visited_packages = HashSet::new();
    // maintain where branches are
    let mut levels = Vec::new();

    print_tree(
        graph,
        package_id,
        symbols,
        &mut visited_packages,
        &mut levels,
    );
}

/// print a package and it's details
fn print_package<'package>(
    _graph: &'package Graph,
    package_id: &'package PackageId,
    visited_packages: &mut HashSet<&'package PackageId>,
) -> bool {
    use crossterm::style::Colorize;

    // insert returns false when they key already exists
    let visited = !visited_packages.insert(package_id);
    let star = if visited { " (*)" } else { "" };

    println!("{}{star}", package_id.to_string().green());

    visited
}

/// print the tree's branches
fn print_branches(levels: &mut Vec<bool>, symbols: &Symbols) {
    if let Some((last, rest)) = levels.split_last() {
        for branch in rest {
            let character = if *branch { symbols.down } else { " " };
            print!("{}   ", character);
        }

        let character = if *last { symbols.tee } else { symbols.ell };
        print!("{0}{1}{1} ", character, symbols.right);
    }
}

/// print a dependency tree
fn print_tree<'package>(
    graph: &'package Graph,
    package_id: &'package PackageId,
    symbols: &Symbols,
    visited_packages: &mut HashSet<&'package PackageId>,
    levels: &mut Vec<bool>,
) {
    if let Some((_package, relationships)) = graph.get(package_id) {
        print_branches(levels, symbols);

        let visited = print_package(graph, package_id, visited_packages);

        // don't recursively enumerate dependencies
        if visited {
            return;
        }

        // zero dependencies means we needn't print anything
        if relationships.is_empty() {
            return;
        }

        for (index, (package_id, _relationship)) in relationships.iter().enumerate() {
            // the last package is the tail
            // inbetween is either a tee or a down
            let is_last = index == relationships.len() - 1;

            levels.push(!is_last);
            print_tree(graph, package_id, symbols, visited_packages, levels);
            levels.pop();
        }
    } else {
        println!("error: missing {package_id}");
    }
}

pub async fn fetch(fetch: Fetch, _http: &reqwest::Client) -> anyhow::Result<()> {
    let packages = Path::new(PREFIX).join("repository");

    if !packages.exists() {
        println!();
        println!("==> \x1b[38;5;9mERROR:\x1b[m repository is missing, did you sync?");
        return Ok(());
    }

    let graph = Graph::open(&packages).await?;

    for package_id in fetch.packages.into_iter().map(PackageId::new) {
        print(&graph, &package_id, &UTF8_SYMBOLS);
    }

    Ok(())
}
