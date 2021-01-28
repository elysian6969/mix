#![feature(format_args_capture)]
#![feature(iter_partition_in_place)]
#![feature(str_split_once)]

use args::Args;
use clap::Clap;

pub mod action;
pub mod args;
pub mod git;
pub mod github;
pub mod source;
pub mod version;

pub const DISTRO: &str = "tiramisu";
pub const DISTRO_VERSION: &str = "0.0.6";
pub const PREFIX: &str = "/tiramisu";
pub const REPOSITORY: &str = "https://github.com/dysmal/mochis";
pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let http = reqwest::Client::builder().user_agent(USER_AGENT).build()?;

    match Args::parse() {
        Args::Fetch(fetch) => action::fetch(fetch, &http).await?,
        Args::Install(install) => action::install(install).await?,
        Args::Remove(remove) => action::remove(remove).await?,
        Args::Sync(sync) => action::sync(sync).await?,
        Args::Update(update) => action::update(update).await?,
    }

    Ok(())
}
