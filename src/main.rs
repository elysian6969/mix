use {
    self::{config::Config, fetch::Client, triple::Triple},
    clap::Clap,
    semver::Version,
    std::{fs::File, iter, path::PathBuf},
};

pub mod config;
pub mod delete_on_drop;
pub mod fetch;
pub mod triple;

#[derive(Clap, Debug)]
pub enum Args {
    Build(Build),
}

#[derive(Clap, Debug)]
pub struct Build {
    #[clap(parse(from_os_str))]
    packages: Vec<PathBuf>,
}

pub mod build {
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
                        crate::fetch::github::fetch_github_tags(&client, &name, &user, &repo)
                            .await?;

                    let avail: BTreeMap<_, _> = avail.into_iter().collect();
                    let (latest_version, _) = avail.iter().rev().next().unwrap();

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
                }
                _ => Err(anyhow::anyhow!("invalid source"))?,
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = Config::new("/tiramisu", iter::once(Triple::default()));
    let client = Client::with_cache("/tiramisu/cache")?;
    let current_version = Version::parse("5.0.0")?;

    match &args {
        Args::Build(build) => {
            for package in &build.packages {
                let script = File::open(&package).map(serde_yaml::from_reader)??;

                for target in config.targets() {
                    build::build(
                        &package,
                        &script,
                        &config,
                        &target,
                        &client,
                        &current_version,
                    )
                    .await?;
                }
            }
        }
    }

    Ok(())
}
