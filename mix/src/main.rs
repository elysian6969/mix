// NOTE: rustc oh phone is old
#![allow(stable_features)]
#![feature(bindings_after_at)]
#![feature(command_access)]
#![feature(core_intrinsics)]
#![feature(crate_visibility_modifier)]
#![feature(drain_filter)]
#![feature(format_args_capture)]
#![feature(map_first_last)]
#![feature(never_type)]
#![feature(option_result_unwrap_unchecked)]
#![feature(str_split_once)]

pub mod atom;
pub mod config;
pub mod external;
pub mod global;
pub mod ops;
pub mod options;
pub mod package;
pub mod partial;
pub mod shell;
pub mod source;
pub mod util;
pub mod version;

pub const fn user_agent() -> &'static str {
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"))
}

use crate::config::Config;
use crate::options::Options;
use std::path::Path;
//use crate::shell::{ProgressBar, Shell, Text};
//use tokio::time::{sleep, Duration};

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T> = std::result::Result<T, Error>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> crate::Result<()> {
    better_panic::install();

    let prefix = Path::new("/milk");
    let metadata = global::Metadata::open(&prefix.join("metadata.yml"))
        .await
        .unwrap();
    let config = Config::builder(prefix)
        .repositories(metadata.repositories)
        .build()?;

    let options = Options::from_env(&config).await.unwrap();

    match options {
        Options::Depend { atoms } => ops::depend(&config, atoms).await.unwrap(),
        Options::Fetch { sync: true } => ops::sync(&config).await.unwrap(),
        Options::Install { atoms } => ops::install(&config, atoms).await.unwrap(),
        _ => {}
    }

    Ok(())
}
