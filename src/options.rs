use clap::Clap;

#[derive(Clap)]
pub enum Options {
    /// Install packages
    Install,

    /// Search packages
    Search(Search),

    /// Uninstall packages
    Uninstall,
}

#[derive(Clap)]
pub struct Search {
    /// Search installed packages
    #[clap(long, short)]
    installed: bool,

    /// Searched not installed packages
    #[clap(long, short)]
    uninstalled: bool,
}

impl Options {
    pub fn parse() -> Self {
        <Self as Clap>::parse()
    }
}
