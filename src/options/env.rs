use clap::Clap;
use milk_atom::Atom;
use milk_env::Config;
use milk_id::{PackageId, RepositoryId};
use milk_triple::Triple;
use path::PathBuf;
use std::str::FromStr;

#[derive(Clap, Debug)]
pub struct Options {
    /// Prefix directory.
    #[clap(default_value = "/milk", long, parse(from_os_str))]
    pub prefix: PathBuf,

    /// Target triple.
    #[clap(default_value = Triple::host().as_str(), long)]
    pub target: Triple,

    /// Package to inspect.
    pub atom: Atom,
}

impl Options {
    pub fn parse() -> Self {
        <Self as Clap>::parse()
    }

    pub fn into_config(self) -> Config {
        Config {
            prefix: self.prefix,
            target: self.target,
            atom: self.atom,
        }
    }
}
