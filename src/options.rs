use clap::{AppSettings, Clap};

mod build;

#[derive(Clap, Debug)]
#[clap(setting = AppSettings::ColoredHelp)]
pub enum Options {
    /// Build a package.
    Build(build::Options),
}

impl Options {
    pub fn parse() -> Self {
        <Self as Clap>::parse()
    }
}
