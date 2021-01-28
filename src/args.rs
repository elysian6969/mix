use clap::Clap;

#[derive(Clap, Debug)]
pub enum Args {
    /// fetch packages' sources
    Fetch(Fetch),

    /// install packages
    Install(Install),

    /// remove packages
    Remove(Remove),

    /// sync repositories
    Sync(Sync),

    /// update packages
    Update(Update),
}

#[derive(Clap, Debug)]
pub struct Fetch {
    pub packages: Vec<String>,
}

#[derive(Clap, Debug)]
pub struct Install {
    #[clap(long)]
    pub target: Option<String>,
    pub packages: Vec<String>,
}

#[derive(Clap, Debug)]
pub struct Remove {
    pub packages: Vec<String>,
}

#[derive(Clap, Debug)]
pub struct Sync {
    pub repos: Vec<String>,
}

#[derive(Clap, Debug)]
pub struct Update {
    pub packages: Vec<String>,
}
