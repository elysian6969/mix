#![feature(crate_visibility_modifier)]
#![feature(format_args_capture)]
#![feature(never_type)]

pub mod atom;
pub mod git;
pub mod github;
pub mod ops;
pub mod options;
pub mod package;
pub mod shell;
pub mod source;
pub mod util;
pub mod version;

pub const DISTRO: &str = "saraphiem";
pub const DISTRO_VERSION: &str = "0.0.6";
pub const PREFIX: &str = "/saraphiem";
pub const REPOSITORY: &str = "https://github.com/dysmal/mochis";
pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

use crate::options::Options;
use crate::shell::Shell;
//use crate::shell::{ProgressBar, Shell, Text};
//use tokio::time::{sleep, Duration};

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T> = std::result::Result<T, Error>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let shell = Shell::new();

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

    let options = Options::from_env(&shell).await?;

    match options {
        Options::Depend { atoms } => ops::depend(&shell, atoms).await?,
        Options::Fetch { sync: true } => ops::sync(&shell).await?,
        _ => {}
    }

    Ok(())
}
