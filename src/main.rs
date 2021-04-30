#![feature(crate_visibility_modifier)]
#![feature(format_args_capture)]
#![feature(str_split_once)]

pub mod action;
pub mod args;
pub mod git;
pub mod github;
pub mod options;
pub mod package;
pub mod source;
pub mod util;
pub mod version;

pub const DISTRO: &str = "saraphiem";
pub const DISTRO_VERSION: &str = "0.0.6";
pub const PREFIX: &str = "/saraphiem";
pub const REPOSITORY: &str = "https://github.com/dysmal/mochis";
pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

use crate::options::Options;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let _options = Options::from_env()?;

    Ok(())
}
