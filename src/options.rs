use clap::{AppSettings, Clap};
use path::{Path, PathBuf};

mod build;
mod env;

#[derive(Clap, Debug)]
pub enum Subcommand {
    /// Inspect a package's build environment.
    Env(env::Options),

    /// Build a package.
    Build(build::Options),
}

#[derive(Clap, Debug)]
#[clap(global_setting = AppSettings::ColoredHelp)]
pub struct Options {
    #[clap(default_value = "/milk", long)]
    pub prefix: PathBuf,

    #[clap(subcommand)]
    pub subcommand: Subcommand,
}

impl Options {
    pub fn parse() -> Self {
        <Self as Clap>::parse()
    }
}
