use crate::options::{Options, Subcommand};
use milk_config::Config;
use milk_packages::Packages;
use tokio::runtime::Builder;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

mod options;

async fn async_main() -> Result<()> {
    let options = Options::parse();
    let config = Config::new(&options.prefix);

    let packages = Packages::from_config(&config).await?;

    println!("{:?}", &packages);

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
