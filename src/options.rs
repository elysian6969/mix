use clap::Parser;
use path::PathBuf;

mod add;
mod remove;
mod sync;

#[derive(Parser, Debug)]
pub enum Subcommand {
    /// install package(s)
    #[clap(alias = "a")]
    Add(add::Options),

    /// remove package(s)
    #[clap(alias = "r")]
    Remove(remove::Options),

    /// sync repos
    #[clap(alias = "s")]
    Sync(sync::Options),
}

/// milk package mangler
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
