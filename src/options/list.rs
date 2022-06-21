use clap::Parser;
use path::PathBuf;

#[derive(Parser, Debug)]
pub struct Options {
    /// Prefix directory.
    #[clap(default_value = "/milk", long, parse(from_os_str))]
    pub prefix: PathBuf,

    /// List installed only.
    #[clap(long, short)]
    pub installed: bool,

    /// Show dependencies.
    #[clap(long, short)]
    pub depend: bool,
}
