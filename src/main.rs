#![feature(format_args_nl)]

use crate::options::{Options, Subcommand};
use mix_config::Config;
use mix_packages::Packages;
use mix_shell::{header, writeln, AsyncDisplay, AsyncWrite};
use std::sync::Arc;
use tokio::runtime::Builder;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

mod options;

async fn async_main() -> Result<()> {
    let options = Options::parse();
    let config = Config::new(&options.prefix).await?;
    let packages = Arc::new(Packages::from_config(&config).await?);

    match options.subcommand {
        Subcommand::Env(env) => {
            mix_env::env(config.clone(), env.into_config()).await?;
        }
        Subcommand::List(list) => {
            for package in packages.iter() {
                if list.installed && !package.installed() {
                    continue;
                }

                header!(
                    config.shell(),
                    "{}/{}",
                    config
                        .shell()
                        .theme()
                        .arguments_paint(package.repository_id()),
                    config.shell().theme().arguments_paint(package.package_id()),
                )?;

                if !package.dependencies().is_empty() {
                    writeln!(config.shell(), "     dependencies")?;

                    for dependency in package.dependencies() {
                        writeln!(config.shell(), "      - {}", dependency)?;
                    }
                }

                if !package.versions().is_empty() {
                    writeln!(config.shell(), "     installed versions")?;

                    for (version, path) in package.versions().pairs() {
                        writeln!(
                            config.shell(),
                            "      - {} ({})",
                            version,
                            config.shell().theme().arguments_paint(path),
                        )?;
                    }
                }

                if package.sources().is_empty() {
                    writeln!(config.shell(), "    no sources (orphan package)")?;
                } else {
                    writeln!(config.shell(), "    sources")?;

                    for source in package.sources().iter() {
                        config.shell().write_str("      - ").await?;
                        source.fmt(config.shell()).await?;
                        config.shell().write_str(" (").await?;
                        source.url().fmt(config.shell()).await?;
                        config.shell().write_str(")\n").await?;
                    }
                }
            }

            writeln!(
                config.shell(),
                "{} total installed packages",
                config
                    .shell()
                    .theme()
                    .arguments_paint(packages.installed().len()),
            )?;
        }
        Subcommand::Build(build) => {
            mix_build::build(config.clone(), build.into_config(), packages.clone()).await?;
        }
        Subcommand::Sync(sync) => {
            mix_sync::sync(config.clone(), sync.into_config()).await?;
        }
    }

    config.shell().flush().await?;

    Ok(())
}

fn main() -> Result<()> {
    Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async_main())
}
