#[derive(Debug)]
pub enum Args {
    Add(Add),
    Del(Del),
    Deps(Deps),
    Fetch(Fetch),
    Sync(Sync),
    Up(Up),
}

/// install packages
#[derive(Debug)]
pub struct Add {
    pub target: Option<String>,
    pub packages: Vec<String>,
}

/// remove packages
#[derive(Debug)]
pub struct Del {
    pub target: Option<String>,
    pub packages: Vec<String>,
}

/// display dependencies of a package
#[derive(Debug)]
pub struct Deps {
    pub package: String,
}

/// download package sources
#[derive(Debug)]
pub struct Fetch {
    pub packages: Vec<String>,
}

/// sync repository
#[derive(Debug)]
pub struct Sync;

/// update packages
#[derive(Debug)]
pub struct Up {
    pub packages: Vec<String>,
}
