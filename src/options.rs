use clap::{AppSettings, Parser};
use path::PathBuf;

mod build;
mod env;
mod list;
mod sync;

#[derive(Parser, Debug)]
pub enum Subcommand {
    /// Inspect a package's build environment.
    Env(env::Options),

    /// List packages.
    List(list::Options),

    /// Build a package.
    Build(build::Options),

    /// Sync repositories
    Sync(sync::Options),
}

#[derive(Parser, Debug)]
pub struct Options {
    #[clap(default_value = "/milk", long)]
    pub prefix: PathBuf,

    #[clap(subcommand)]
    pub subcommand: Subcommand,
}

impl Options {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
