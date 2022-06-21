use clap::Parser;
use mix_atom::Requirement;
use mix_build::{Config, Value};
use mix_triple::Triple;
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

#[derive(Parser, Debug)]
pub struct Options {
    /// Prefix directory.
    #[clap(default_value = "/milk", long, parse(from_os_str))]
    pub prefix: PathBuf,

    /// Target triple.
    #[clap(default_value = Triple::host().as_str(), long)]
    pub target: Triple,

    /// Package to install.
    pub requirement: Requirement,

    /// Jobs to build with.
    #[clap(default_value = "1", long, short)]
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

impl Into<Config> for Options {
    #[inline]
    fn into(self) -> Config {
        Config {
            prefix: self.prefix,
            target: self.target,
            requirement: self.requirement,
            jobs: self.jobs,
            define: self.define,
            include: self.include,
            build_dir: self.build_dir,
        }
    }
}
