#![feature(format_args_nl)]

use crate::options::{Options, Subcommand};
use mix_config::Config;
use mix_packages::Packages;
use mix_shell::{header, writeln, AsyncDisplay, AsyncWrite};
use tokio::runtime::Builder;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

mod options;

async fn async_main() -> Result<()> {
    let options = Options::parse();
    let config = Config::new(&options.prefix);
    let packages = Packages::from_config(&config).await?;
    let mut installed = 0_usize;

    for package in packages.iter() {
        header!(
            config.shell(),
            "{}/{}",
            config
                .shell()
                .theme()
                .arguments_paint(package.repository_id()),
            config.shell().theme().arguments_paint(package.package_id()),
        )?;

        // TODO: add a newtype
        if package.dependencies().len() > 0 {
            writeln!(config.shell(), "     dependencies")?;

            for dependency in package.dependencies() {
                writeln!(config.shell(), "      - {}", dependency)?;
            }
        }

        // TODO: add a newtype
        if package.version_pairs().len() > 0 {
            writeln!(config.shell(), "     installed versions")?;

            for (version, path) in package.version_pairs().iter() {
                writeln!(
                    config.shell(),
                    "      - {} ({})",
                    version,
                    config.shell().theme().arguments_paint(path),
                )?;

                installed += 1;
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
        config.shell().theme().arguments_paint(installed),
    )?;

    config.shell().flush().await?;

    match options.subcommand {
        Subcommand::Env(env) => {
            mix_env::env(config, env.into_config()).await?;
        }
        Subcommand::Build(build) => {
            mix_build::build(config, build.into_config()).await?;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async_main())
}
