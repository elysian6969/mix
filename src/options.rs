use clap::{AppSettings, Clap};

mod build;
mod env;

#[derive(Clap, Debug)]
#[clap(setting = AppSettings::ColoredHelp)]
pub enum Options {
    /// Inspect a package's build environment.
    Env(env::Options),

    /// Build a package.
    Build(build::Options),
}

impl Options {
    pub fn parse() -> Self {
        <Self as Clap>::parse()
    }
}
