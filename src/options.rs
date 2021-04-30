use crossterm::style::Colorize;

#[derive(Debug)]
pub struct Options {}

impl Options {
    pub fn from_env() -> Result<Options, pico_args::Error> {
        let mut args = pico_args::Arguments::from_env();

        if args.contains(["-h", "--help"]) {
            println!(
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

            std::process::exit(0);
        }

        Ok(Options {})
    }
}
