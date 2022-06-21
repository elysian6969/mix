#![feature(format_args_nl)]

use crate::options::{Options, Subcommand};
use mix_config::Config;
use mix_packages::Packages;
use mix_shell::{header, writeln, AsyncDisplay, AsyncWrite};
use std::sync::Arc;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

mod options;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let options = Options::parse();
    let config = Config::new(&options.prefix).await?;
    let packages = Arc::new(Packages::from_config(&config).await?);

    match options.subcommand {
        Subcommand::Add(add) => {
            mix_build::build(config.clone(), add.into(), packages.clone()).await?;
        }
        Subcommand::Sync(sync) => {
            mix_sync::sync(config.clone(), sync.into()).await?;
        }
        _ => todo!(),
    }

    config.shell().flush().await?;

    Ok(())
}
