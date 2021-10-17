use clap::Clap;
use path::PathBuf;

#[derive(Clap, Debug)]
pub struct Options {
    /// Prefix directory.
    #[clap(default_value = "/mix", long, parse(from_os_str))]
    pub prefix: PathBuf,

    /// List installed only.
    #[clap(long, short)]
    pub installed: bool,
}
