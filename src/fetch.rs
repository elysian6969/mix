use super::args::{Fetch, Install, Remove, Sync, Update};
use super::PREFIX;

pub async fn fetch(mut fetch: Fetch) -> anyhow::Result<()> {
    use std::collections::BTreeSet;
    use std::fs;
    use std::fs::File;
    use std::iter::FromIterator;
    use std::path::Path;

    println!(" -> read repository");

    if !Path::new("/tiramisu/mochis").exists() {
        println!();
        eprintln!("==> \x1b[38;5;9mERROR:\x1b[m repository is missing, did you sync?");
        return Ok(());
    }

    let repo = Path::new("/tiramisu/mochis/src");
    let i = fetch
        .packages
        .iter_mut()
        .partition_in_place(|p| repo.join(p).exists());

    let existing = &fetch.packages[..i];
    let missing = &fetch.packages[i..];

    if missing.len() != 0 {
        println!();
        eprintln!(
            "==> \x1b[38;5;11mWARNING:\x1b[m \"{}\" not found in repository",
            missing.join("\", \"")
        );
    }

    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct Hint {
        source: Vec<String>,
    }

    for p in existing.iter() {
        let path = repo.join(p).join("package.yml");
        let hint: Hint = serde_yaml::from_reader(File::open(&path)?)?;

        let iter = hint.source.iter().map(|h| h.split_once(':'));

        println!("{path:?}");
        println!("{hint:?}");
        println!("{iter:?}");
    }

    Ok(())
}

pub async fn install(install: Install) -> anyhow::Result<()> {
    Ok(())
}

pub async fn remove(remove: Remove) -> anyhow::Result<()> {
    Ok(())
}

pub async fn sync(sync: Sync) -> anyhow::Result<()> {
    use std::path::Path;
    use tokio::process::Command;

    if Path::new("/tiramisu/mochis").exists() {
        println!(" -> fetch");

        Command::new("git")
            .arg("fetch")
            .arg("--depth=1")
            .current_dir("/tiramisu/mochis")
            .spawn()?
            .wait()
            .await?;

        println!(" -> fast-forward");
        Command::new("git")
            .arg("merge")
            .arg("--ff-only")
            .current_dir("/tiramisu/mochis")
            .spawn()?
            .wait()
            .await?;
    } else {
        println!(" -> clone");

        Command::new("git")
            .arg("clone")
            .arg("--depth=1")
            .arg("https://github.com/tatenashii/mochis.git")
            .arg("/tiramisu/mochis")
            .spawn()?
            .wait()
            .await?;
    }

    Ok(())
}

pub async fn update(update: Update) -> anyhow::Result<()> {
    Ok(())
}
