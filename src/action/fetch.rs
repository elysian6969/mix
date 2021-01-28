use crate::args::Fetch;
use crate::github;
use crate::source::Source;
use crate::PREFIX;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Specification {
    source: BTreeSet<Source>,
}

pub async fn fetch(fetch: Fetch, http: &reqwest::Client) -> anyhow::Result<()> {
    let prefix = Path::new(PREFIX);
    let packages = prefix.join("repository");

    println!(" -> read repository");

    if !packages.exists() {
        println!();
        println!("==> \x1b[38;5;9mERROR:\x1b[m repository is missing, did you sync?");
        return Ok(());
    }

    let list = packages.join("src");

    let (existing, missing): (BTreeMap<_, _>, BTreeMap<_, _>) = fetch
        .packages
        .into_iter()
        .map(|name| {
            let path = list.join(&name).join("package.yml");

            (name, path)
        })
        .partition(|(name, path)| path.exists());

    if missing.len() != 0 {
        println!();
        eprintln!("==> \x1b[38;5;11mWARNING:\x1b[m not found in repository",);
    }

    for (name, path) in &existing {
        println!(" -> parsing package `{name}`");

        let specification: Specification = serde_yaml::from_reader(File::open(&path)?)?;

        for source in &specification.source {
            match source {
                Source::Github(user, repository) => {
                    let tags = github::tags(&http, user, repository).await?;

                    println!();

                    for (version, url) in tags {
                        print!("    {version}");

                        if !version.pre.is_empty() {
                            print!(" \x1b[38;5;9munstable\x1b[m");
                        }

                        println!();
                    }

                    println!();
                }
            }
        }
    }

    Ok(())
}
