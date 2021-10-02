use clap::Clap;
use milk_atom::Atom;
use milk_build::{Config, Value};
use milk_id::{PackageId, RepositoryId};
use milk_triple::Triple;
use path::PathBuf;
use std::str::FromStr;

fn parse_key_val<'s, T>(s: &'s str) -> crate::Result<(T, Value)>
where
    T: From<&'s str>,
{
    let (k, v) = s
        .split_once('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;

    let k = k.into();
    let v = bool::from_str(v)
        .map(Value::Bool)
        .ok()
        .or_else(|| Some(Value::String(v.into())))
        .unwrap();

    Ok((k, v))
}

#[derive(Clap, Debug)]
pub struct Options {
    /// Prefix directory.
    #[clap(default_value = "/milk", long, parse(from_os_str))]
    pub prefix: PathBuf,

    /// Target triple.
    #[clap(default_value = Triple::host().as_str(), long)]
    pub target: Triple,

    /// Package to install.
    pub atom: Atom,

    /// Jobs to build with.
    #[clap(long, short)]
    pub jobs: usize,

    /// Maps to `--enable/--disable`.
    #[clap(long, multiple_occurrences = true, parse(try_from_str = parse_key_val), short = 'D')]
    pub define: Vec<(String, Value)>,

    /// Maps to `--with/--without`.
    #[clap(long, multiple_occurrences = true, parse(try_from_str = parse_key_val), short = 'I')]
    pub include: Vec<(String, Value)>,

    /// Whether this package requires a seperate build directory.
    #[clap(long)]
    pub build_dir: bool,
}

impl Options {
    pub fn parse() -> Self {
        <Self as Clap>::parse()
    }

    pub fn into_config(self) -> Config {
        Config {
            prefix: self.prefix,
            target: self.target,
            atom: self.atom,
            jobs: self.jobs,
            define: self.define,
            include: self.include,
            build_dir: self.build_dir,
        }
    }
}
