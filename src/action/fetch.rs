use crate::args::Fetch;
use crate::github;
use crate::package::{Package, Repository};
use crate::source::Source;
use crate::PREFIX;
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::path::Path;
use std::rc::{Rc, Weak};

fn print_package(depth: usize, package: &Weak<Package>) {
    if package.strong_count() > 1 {
        println!("cyclic dependencies!");
        return;
    }

    println!("{} {}", package.strong_count(), package.weak_count());

    if let Some(package) = package.upgrade() {
        for (name, package) in &*package.depends() {
            println!("{depth} {name}");

            print_package(depth + 1, package);
        }
    }
}

pub async fn fetch(fetch: Fetch, http: &reqwest::Client) -> anyhow::Result<()> {
    let prefix = Path::new(PREFIX);
    let packages = prefix.join("repository");

    if !packages.exists() {
        println!();
        println!("==> \x1b[38;5;9mERROR:\x1b[m repository is missing, did you sync?");
        return Ok(());
    }

    let repository = Repository::open(&packages).await?;

    for name in &fetch.packages {
        if let Some(package) = repository.get(&name) {
            print_package(0, &Rc::downgrade(&package));
        }
    }

    Ok(())
}
