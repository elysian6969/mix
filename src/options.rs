use crate::atom::Atom;
use crate::shell::{Shell, Text};
use crossterm::style::Colorize;
use pico_args::Arguments;
use std::collections::HashSet;

#[derive(Debug)]
pub enum Options {
    Depend { atoms: HashSet<Atom> },
    Fetch { sync: bool },
    Install { atoms: HashSet<Atom> },
    Remove { atoms: HashSet<Atom> },
    Search,
}

impl Options {
    pub async fn from_env(shell: &Shell) -> crate::Result<Options> {
        let mut args = Arguments::from_env();

        if args.contains(["-h", "--help"]) {
            print_help(&shell).await?;
        }

        let subcommand = match args.subcommand()? {
            Some(subcommand) => subcommand,
            None => print_help(&shell).await?,
        };

        let options = match subcommand.as_str() {
            "d" | "depend" => Options::Depend {
                atoms: into_atoms(args),
            },
            "f" | "fetch" => Options::Fetch {
                sync: args.contains(["-s", "--sync"]),
            },
            "i" | "install" => Options::Install {
                atoms: into_atoms(args),
            },
            "r" | "remove" => Options::Remove {
                atoms: into_atoms(args),
            },
            "s" | "search" => Options::Search,
            _ => print_help(&shell).await?,
        };

        Ok(options)
    }
}

fn into_atoms(arguments: Arguments) -> HashSet<Atom> {
    arguments
        .finish()
        .iter()
        .flat_map(|atom| Atom::parse(atom.to_str()?).ok())
        .collect()
}

async fn print_help(shell: &Shell) -> crate::Result<!> {
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

    Text::new(format).render(&shell).await?;
    std::process::exit(0)
}