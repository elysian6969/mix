use clap::Clap;
use mix_atom::Atom;
use mix_env::Config;
use mix_triple::Triple;
use path::PathBuf;

#[derive(Clap, Debug)]
pub struct Options {
    /// Prefix directory.
    #[clap(default_value = "/mix", long, parse(from_os_str))]
    pub prefix: PathBuf,

    /// Target triple.
    #[clap(default_value = Triple::host().as_str(), long)]
    pub target: Triple,

    /// Package to inspect.
    pub atom: Atom,
}

impl Options {
    pub fn into_config(self) -> Config {
        Config {
            prefix: self.prefix,
            target: self.target,
            atom: self.atom,
        }
    }
}
