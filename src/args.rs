use clap::Clap;

#[derive(Clap, Debug)]
pub enum Args {
    /// install packages
    Add(Add),

    /// remove packages
    Del(Del),

    /// see what a package depends om
    Depends(Depends),

    /// fetch packages' sources
    Fetch(Fetch),

    /// sync repositories
    Sync(Sync),

    /// update packages
    Update(Update),
}

#[derive(Clap, Debug)]
pub struct Add {
    #[clap(long)]
    pub target: Option<String>,
    pub packages: Vec<String>,
}

#[derive(Clap, Debug)]
pub struct Del {
    pub packages: Vec<String>,
}

#[derive(Clap, Debug)]
pub struct Depends {
    pub packages: Vec<String>,
}

#[derive(Clap, Debug)]
pub struct Fetch {
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
