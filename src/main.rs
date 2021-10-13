use crate::options::{Options, Subcommand};
use milk_config::Config;
use milk_packages::Packages;
use milk_shell::{write, AsyncWrite};
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
        write!(
            config.shell(),
            "> {}/{}\n",
            package.repository_id(),
            package.package_id()
        )
        .await?;

        // TODO: add a newtype
        if package.dependencies().len() > 0 {
            write!(config.shell(), "   Dependencies\n").await?;

            for dependency in package.dependencies() {
                write!(config.shell(), "    - {}\n", dependency).await?;
            }
        }

        // TODO: add a newtype
        if package.version_pairs().len() > 0 {
            write!(config.shell(), "   Installed versions\n").await?;

            for (version, path) in package.version_pairs().iter() {
                write!(config.shell(), "    - {} ({})\n", version, path).await?;

                installed += 1;
            }
        }

        if package.sources().len() > 0 {
            write!(config.shell(), "   Sources\n").await?;

            for source in package.sources().iter() {
                write!(config.shell(), "    - {} ({})\n", source, source.url()).await?;
            }
        } else {
            write!(config.shell(), "  No sources (orphan package)\n").await?;
        }
    }

    write!(config.shell(), "{} total installed packages.\n", installed).await?;
    config.shell().flush().await?;

    match options.subcommand {
        Subcommand::Env(env) => {
            milk_env::env(config, env.into_config()).await?;
        }
        Subcommand::Build(build) => {
            milk_build::build(config, build.into_config()).await?;
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
