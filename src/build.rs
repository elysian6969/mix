use super::config::Config;
use super::delete_on_drop::DeleteOnDrop;
use super::fetch::Client;
use super::triple::Triple;
use crossterm::style::{Colorize, Styler};
use multimap::MultiMap;
use semver::Version;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use tokio::{fs, process::Command};
use url::Url;

#[derive(Debug, Deserialize)]
pub struct Script {
    pub source: Vec<Url>,
    pub configure: Option<Vec<String>>,
    pub make: Option<Vec<String>>,
}

pub async fn build(
    path: &PathBuf,
    script: &Script,
    config: &Config,
    triple: &Triple,
    client: &Client,
    current_version: &Version,
) -> anyhow::Result<()> {
    let name = path
        .file_stem()
        .and_then(|string| string.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid name"))?;

    for source in &script.source {
        match source.scheme() {
            "github" => {
                let mut segments = source.path().split('/');

                let user = segments
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("invalid github user"))?;

                let repo = segments
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("invalid github repo"))?;

                let avail =
                    crate::fetch::github::fetch_github_tags(&client, &name, &user, &repo).await?;

                let avail: BTreeMap<_, _> = avail.into_iter().collect();
                let (latest_version, latest) = avail.iter().rev().next().unwrap();

                if latest_version > &current_version {
                    println!(
                            "{package}{colon} update available {current_version} {arrow} {latest_version}",
                            arrow = "->".bold(),
                            colon = ":".bold(),
                            current_version = current_version.to_string().dark_red().bold(),
                            latest_version = latest_version.to_string().dark_green().bold(),
                            package = name.bold(),
                        );
                }

                let combined = format!("{}-{}", &name, &latest_version);
                let tarball = format!("{}.tar.xz", &combined);

                client
                    .get_partial(&name, &tarball, &latest.tarball_url)
                    .await?;

                let target_dir = config.prefix().join(triple.to_string()).join(&combined);

                if target_dir.exists() {
                    println!("{}: already installed", &combined);

                    return Ok(());
                }

                let build_dir = config.build().join(&combined);

                if build_dir.exists() {
                    println!(
                        "{}: skipping this package, it's being built by another instance",
                        &combined
                    );

                    return Ok(());
                }

                fs::create_dir_all(&build_dir).await?;

                let _guard = DeleteOnDrop::new(&build_dir);
                let full_tarball_path = client.cache().join(&name).join(&tarball);

                Command::new("tar")
                    .current_dir(&build_dir)
                    .arg("pxf")
                    .arg(&full_tarball_path)
                    .spawn()?
                    .await?;

                let entries: MultiMap<String, DirEntry> = WalkDir::new(&build_dir)
                    .into_iter()
                    .flat_map(|entry| {
                        let entry = entry.ok()?;
                        let file_name = entry.path().file_name()?.to_str()?;

                        Some((file_name.to_string(), entry))
                    })
                    .collect();

                if let Some(configures) = entries.get_vec("configure") {
                    let roots: HashMap<_, _> = configures
                        .iter()
                        .flat_map(|path| Some((path.path().parent()?, path)))
                        .collect();

                    let (root, configure) = roots.iter().next().unwrap();

                    Command::new(configure.path())
                        .arg(format!(
                            "--prefix={}",
                            config
                                .prefix()
                                .join(triple.to_string())
                                .join(&combined)
                                .display()
                        ))
                        .current_dir(&root)
                        .env_clear()
                        .env("CFLAGS", "-Ofast -march=znver2 -pipe")
                        .env("CXXFLAGS", "-Ofast -march=znver2 -pipe")
                        .env("LANG", "C.UTF-8")
                        .env("PATH", "/bin:/sbin")
                        .env("TERM", "linux")
                        .spawn()?
                        .await?;

                    Command::new("make")
                        .current_dir(&root)
                        .env_clear()
                        .env("CFLAGS", "-Ofast -march=znver2 -pipe")
                        .env("CXXFLAGS", "-Ofast -march=znver2 -pipe")
                        .env("LANG", "C.UTF-8")
                        .env("PATH", "/bin:/sbin")
                        .env("TERM", "linux")
                        .spawn()?
                        .await?;

                    Command::new("make")
                        .arg("install")
                        .current_dir(&root)
                        .env_clear()
                        .env("CFLAGS", "-Ofast -march=znver2 -pipe")
                        .env("CXXFLAGS", "-Ofast -march=znver2 -pipe")
                        .env("LANG", "C.UTF-8")
                        .env("PATH", "/bin:/sbin")
                        .env("TERM", "linux")
                        .spawn()?
                        .await?;
                }
            }
            _ => Err(anyhow::anyhow!("invalid source"))?,
        }
    }

    Ok(())
}
