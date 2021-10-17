use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term::{Chars, Config};
use mix_atom::Requirement;
use mix_source::Source;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::str::FromStr;
use std::{error, fmt, io};

#[derive(Debug, Deserialize, Serialize)]
pub struct Manifest {
    #[serde(default, rename = "depend")]
    pub dependencies: BTreeSet<Requirement>,
    #[serde(default, rename = "source")]
    pub sources: BTreeSet<Source>,
}

impl FromStr for Manifest {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let this: Self = serde_yaml::from_str(text)?;

        Ok(this)
    }
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Serde(serde_yaml::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

// TODO: Reconstruct error from display of serde_yaml::Error.
impl From<serde_yaml::Error> for Error {
    fn from(error: serde_yaml::Error) -> Self {
        Error::Serde(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        match &self {
            Io(error) => fmt.write_fmt(format_args!("{}", error))?,
            Serde(error) => fmt.write_fmt(format_args!("{}", error))?,
        }

        Ok(())
    }
}

impl error::Error for Error {}

// TODO: use mix_shell
pub fn print_error(
    error: serde_yaml::Error,
    file_name: &str,
    manifest: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = Config {
        chars: Chars::ascii(),
        ..Config::default()
    };

    let file = SimpleFile::new(file_name, &manifest);
    let start = error.location().unwrap().index();
    let rest = &manifest[start..];
    let end = start + rest.find('\n').unwrap_or(rest.len());
    let message = error.to_string();

    // NOTE: this is utter shit.
    if message.contains("invalid type: map, expected atom requirement") {
        let diagnostic = Diagnostic::error()
            .with_message("failed to parse manifest")
            .with_labels(vec![
                Label::primary((), end..end).with_message("invalid atom, wrap it in \"\"")
            ]);

        term::emit(&mut writer.lock(), &config, &file, &diagnostic)?;
    } else if message.contains("unexpected character in package id") {
        let diagnostic = Diagnostic::error()
            .with_message("failed to parse manifest")
            .with_labels(vec![
                Label::primary((), start..end).with_message("unexpected character in package id")
            ]);

        term::emit(&mut writer.lock(), &config, &file, &diagnostic)?;
    } else if message.contains("unexpected character in repository id") {
        let diagnostic = Diagnostic::error()
            .with_message("failed to parse manifest")
            .with_labels(vec![
                Label::primary((), start..end).with_message("unexpected character in package id")
            ]);

        term::emit(&mut writer.lock(), &config, &file, &diagnostic)?;
    } else if message.contains("invalid type: map, expected source") {
        let diagnostic = Diagnostic::error()
            .with_message("failed to parse manifest")
            .with_labels(vec![
                Label::primary((), end..end).with_message("invalid source, wrap it in \"\"")
            ]);

        term::emit(&mut writer.lock(), &config, &file, &diagnostic)?;
    } else if message.contains("unknown scheme") {
        let diagnostic = Diagnostic::error()
            .with_message("failed to parse manifest")
            .with_labels(vec![
                Label::primary((), start..end).with_message("unknown source scheme")
            ]);

        term::emit(&mut writer.lock(), &config, &file, &diagnostic)?;
    } else if message.contains("expected user") {
        let diagnostic = Diagnostic::error()
            .with_message("failed to parse manifest")
            .with_labels(vec![
                Label::primary((), start..end).with_message("expected user in source")
            ]);

        term::emit(&mut writer.lock(), &config, &file, &diagnostic)?;
    } else {
        println!("DEBUG {:?}", &message);
    }

    Ok(())
}

/*async fn async_main() {
    let path = std::env::args().nth(1).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    let manifest = match Manifest::from_str(&text) {
        Ok(manifest) => manifest,
        Err(Error::Serde(error)) => {
            crate::print_error(error, &path, &text).unwrap();

            return;
        }
        Err(error) => {
            println!("{:?}", error);

            return;
        }
    };

    println!("dependencies");

    for dependency in manifest.dependencies {
        println!("  {:?}", dependency);
    }

    println!("sources");

    for source in manifest.sources {
        println!("  {:?}", source);
        println!("    source url: {}", source.url());
        println!("    cache directory: {}", source.cache("/mix/cache"));
    }
}*/
