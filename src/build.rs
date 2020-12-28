use {
    super::{config::Config, fetch::Client, triple::Triple},
    crossterm::style::{Colorize, Styler},
    semver::Version,
    serde::Deserialize,
    std::{collections::BTreeMap, path::PathBuf},
    url::Url,
};

#[derive(Debug, Deserialize)]
pub struct Script {
    pub source: Vec<Url>,
    pub configure: Option<Vec<String>>,
    pub make: Option<Vec<String>>,
}

pub async fn build(
    path: &PathBuf,
    script: &Script,
    _config: &Config,
    _triple: &Triple,
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

                let tarball = format!("{}-{}.tar.xz", &name, &latest_version);

                client
                    .get_partial(&name, &tarball, &latest.tarball_url)
                    .await?;
            }
            _ => Err(anyhow::anyhow!("invalid source"))?,
        }
    }

    Ok(())
}
