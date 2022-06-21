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
}
