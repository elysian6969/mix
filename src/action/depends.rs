use crate::args::Depends;
use crate::github;
use crate::source::Source;
use crate::PREFIX;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Specification {
    depends: Option<BTreeSet<String>>,
    source: BTreeSet<Source>,
}

pub async fn depends(depends: Depends) -> anyhow::Result<()> {
    let prefix = Path::new(PREFIX);
    let packages = prefix.join("repository");

    println!(" -> read repository");

    if !packages.exists() {
        println!();
        println!("==> \x1b[38;5;9mERROR:\x1b[m repository is missing, did you sync?");
        return Ok(());
    }

    let list = packages.join("src");

    let (existing, missing): (BTreeMap<_, _>, BTreeMap<_, _>) = depends
        .packages
        .into_iter()
        .map(|name| {
            let path = list.join(&name).join("package.yml");

            (name, path)
        })
        .partition(|(name, path)| path.exists());

    if missing.len() != 0 {
        println!();

        let missing: Vec<_> = missing.keys().map(|name| format!("`{name}`")).collect();
        let missing = missing.join(", ");

        println!("==> \x1b[38;5;11mWARNING:\x1b[m {missing} not found in repository");
        println!();
    }

    for (name, path) in &existing {
        println!(" -> parsing package `{name}`");

        let specification: Specification = serde_yaml::from_reader(File::open(&path)?)?;

        if specification.depends.is_none() {
            println!(" -> `{name}` has no dependencies");
        } else {
            let depends: Vec<_> = specification
                .depends
                .iter()
                .flatten()
                .map(|name| format!("`{name}`"))
                .collect();
            let depends = depends.join(", ");

            println!(" -> `{name}` depends on {depends}");
        }
    }

    Ok(())
}
