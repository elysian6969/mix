use clap::{AppSettings, Clap};

#[derive(Clap, Debug)]
#[clap(global_setting = AppSettings::UnifiedHelpMessage)]
pub enum Args {
    Add(Add),
    Del(Del),
    Deps(Deps),
    Fetch(Fetch),
    Sync(Sync),
    Up(Up),
}

/// install packages
#[derive(Clap, Debug)]
pub struct Add {
    #[clap(long)]
    pub target: Option<String>,
    #[clap(min_values = 1, required = true)]
    pub packages: Vec<String>,
}

/// remove packages
#[derive(Clap, Debug)]
pub struct Del {
    #[clap(long)]
    pub target: Option<String>,
    #[clap(min_values = 1, required = true)]
    pub packages: Vec<String>,
}

/// display dependencies of a package
#[derive(Clap, Debug)]
pub struct Deps {
    pub package: String,
}

/// download package sources
#[derive(Clap, Debug)]
pub struct Fetch {
    #[clap(min_values = 1, required = true)]
    pub packages: Vec<String>,
}

/// sync repository
#[derive(Clap, Debug)]
pub struct Sync;

/// update packages
#[derive(Clap, Debug)]
pub struct Up {
    #[clap(min_values = 1, required = true)]
    pub packages: Vec<String>,
}
