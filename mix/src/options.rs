use crate::atom::Atom;
use crate::config::Config;
use crate::shell::Text;
use crossterm::style::Stylize;
use pico_args::Arguments;
use std::collections::HashSet;
use ufmt::derive::uDebug;

#[derive(uDebug)]
pub enum Options {
    Depend { atoms: HashSet<Atom> },
    Fetch { sync: bool },
    Install { atoms: HashSet<Atom> },
    Remove { atoms: HashSet<Atom> },
    Search,
}

impl Options {
    pub async fn from_env(config: &Config) -> crate::Result<Options> {
        let mut args = Arguments::from_env();

        if args.contains(["-h", "--help"]) {
            print_help(&config).await?;
        }

        let subcommand = match args.subcommand()? {
            Some(subcommand) => subcommand,
            None => print_help(&config).await?,
        };

        let options = match subcommand.as_str() {
            "d" | "depend" => Options::Depend {
                atoms: into_atoms(config, args).await,
            },
            "f" | "fetch" => Options::Fetch {
                sync: args.contains(["-s", "--sync"]),
            },
            "i" | "install" => Options::Install {
                atoms: into_atoms(config, args).await,
            },
            "r" | "remove" => Options::Remove {
                atoms: into_atoms(config, args).await,
            },
            "s" | "search" => Options::Search,
            _ => print_help(&config).await?,
        };

        Ok(options)
    }
}

async fn into_atoms(config: &Config, args: Arguments) -> HashSet<Atom> {
    let mut atoms = HashSet::new();

    for arg in args.finish() {
        let arg = arg.to_str().expect("kill yourself");

        match Atom::parse(arg) {
            Ok(atom) => {
                atoms.insert(atom);
            }
            Err(ref error) => {
                let buffer = format!("{error:?}");
                let _ = Text::new(buffer).render(config.shell()).await;
            }
        }
    }

    atoms
}

async fn print_help(config: &Config) -> crate::Result<!> {
    let format = format!(
        "\
usage {program} {arguments}

{arguments}

  {d} {depend}  query dependency information
  {f} {fetch}   download metadata, packages, or a repository
  {i} {install} install packages and repositories
  {r} {remove}  remove packages and repositories
  {s} {search}  search for packages and repositories
",
        program = concat!(" ", env!("CARGO_PKG_NAME"), " ").black().on_green(),
        arguments = " arguments ".black().on_yellow(),
        d = " d ".black().on_yellow(),
        depend = " depend ".black().on_yellow(),
        f = " f ".black().on_yellow(),
        fetch = " fetch ".black().on_yellow(),
        i = " i ".black().on_yellow(),
        install = " install ".black().on_yellow(),
        r = " r ".black().on_yellow(),
        remove = " remove ".black().on_yellow(),
        s = " s ".black().on_yellow(),
        search = " search ".black().on_yellow()
    );

    Text::new(format).render(config.shell()).await?;
    std::process::exit(0)
}
