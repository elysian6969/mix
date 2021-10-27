use clap::Parser;
use mix_id::RepositoryId;
use mix_sync::Config;
use path::PathBuf;

#[derive(Parser, Debug)]
pub struct Options {
    /// Prefix directory.
    #[clap(default_value = "/milk", long, parse(from_os_str))]
    pub prefix: PathBuf,

    /// Repositories to sync.
    pub repositories: Vec<RepositoryId>,
}

impl Options {
    pub fn into_config(self) -> Config {
        Config {
            prefix: self.prefix,
            repositories: self.repositories.into_iter().collect(),
        }
    }
}
