// NOTE: rustc oh phone is old
#![allow(stable_features)]
#![feature(crate_visibility_modifier)]
#![feature(format_args_capture)]
#![feature(never_type)]
#![feature(str_split_once)]

pub mod atom;
pub mod config;
pub mod git;
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
    let prefix = Path::new("/unknown");
    let metadata = global::Metadata::open(&prefix.join("unknown.yml")).await?;
    let config = Config::builder(prefix)
        .repositories(metadata.repositories)
        .build()?;

    /*for progress in 0u32..=100 {
        Text::new(format_args!("{progress:>2}% "))
            .render(&shell)
            .await?;
        ProgressBar::new(0.0..=100.0, progress as f32)
            .render(&shell)
            .await?;
        Text::new("\r").render(&shell).await?;
        sleep(Duration::from_millis(100)).await;
    }

    Text::new("\x1b[K").render(&shell).await?;*/

    let options = Options::from_env(&config).await?;

    match options {
        Options::Depend { atoms } => ops::depend(&config, atoms).await?,
        Options::Fetch { sync: true } => ops::sync(&config).await?,
        Options::Install { atoms } => ops::install(&config, atoms).await?,
        _ => {}
    }

    Ok(())
}
