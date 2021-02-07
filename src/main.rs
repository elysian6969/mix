#![feature(crate_visibility_modifier)]
#![feature(format_args_capture)]
#![feature(str_split_once)]

use args::Args;
use clap::Clap;

pub mod action;
pub mod args;
pub mod git;
pub mod github;
pub mod package;
pub mod source;
pub mod util;
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
        Args::Add(add) => action::add(add).await?,
        Args::Del(del) => action::del(del).await?,
        Args::Deps(deps) => action::deps(deps).await?,
        Args::Fetch(fetch) => action::fetch(fetch, &http).await?,
        Args::Sync(sync) => action::sync(sync).await?,
        Args::Up(up) => action::update(up).await?,
    }

    Ok(())
}
