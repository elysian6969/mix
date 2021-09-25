use atom::AtomReq;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term::{Chars, Config};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::io;
use std::str::FromStr;
use tokio::runtime::Builder;

#[derive(Debug, Deserialize, Serialize)]
pub struct Manifest {
    #[serde(default, rename = "depends")]
    dependencies: BTreeSet<AtomReq>,
    #[serde(default, rename = "source")]
    sources: BTreeSet<String>,
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

fn error(
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

    println!("{:?}", &message);

    if message.contains("invalid type: map, expected atom requirement") {
        let atom = AtomReq::from_str(&manifest[start..end]);
        let diagnostic = Diagnostic::error()
            .with_message("failed to parse manifest")
            .with_labels(vec![
                Label::primary((), end..end).with_message("invalid atom, wrap it in \"\"")
            ]);

        term::emit(&mut writer.lock(), &config, &file, &diagnostic)?;
    }

    if message.contains("unexpected character in package id") {
        let atom = AtomReq::from_str(&manifest[start..end]);
        let diagnostic = Diagnostic::error()
            .with_message("failed to parse manifest")
            .with_labels(vec![
                Label::primary((), start..end).with_message("unexpected character in package id")
            ]);

        term::emit(&mut writer.lock(), &config, &file, &diagnostic)?;
    }

    if message.contains("unexpected character in repository id") {
        let atom = AtomReq::from_str(&manifest[start..end]);
        let diagnostic = Diagnostic::error()
            .with_message("failed to parse manifest")
            .with_labels(vec![
                Label::primary((), start..end).with_message("unexpected character in package id")
            ]);

        term::emit(&mut writer.lock(), &config, &file, &diagnostic)?;
    }

    Ok(())
}

async fn async_main() {
    let path = std::env::args().nth(1).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    let manifest = match Manifest::from_str(&text) {
        Ok(manifest) => manifest,
        Err(Error::Serde(error)) => {
            crate::error(error, &path, &text).unwrap();
            return;
        }
        Err(error) => {
            println!("{:?}", error);
            return;
        }
    };

    println!("dependencies");

    for dependency in manifest.dependencies {
        println!("  {}", dependency);
    }

    println!("sources");

    for source in manifest.sources {
        println!("  {}", source);
    }
}

fn main() {
    Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())
}