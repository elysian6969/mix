use super::process::{Command, Stdio};
use crate::ops::install::build::Build;
use crate::shell::{Colour, Line};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, BufReader};
use ufmt::derive::uDebug;

pub use self::error::{Error, Exit};

mod error;

pub mod lexer;
pub mod parser;

pub struct Autotools<'a> {
    /// `--enable` and `--disable`
    definitions: &'a [Pair<'a>],
    /// `--with` and `--without`
    inclusions: &'a [Pair<'a>],
}

pub struct Pair<'a> {
    key: &'a str,
    val: Value<'a>,
}

impl<'a> Pair<'a> {
    pub fn to_string(&self, inclusions: bool) -> String {
        match (inclusions, &self.val) {
            (false, Value::Bool(false)) => format!("--disable-{}", self.key),
            (false, val) => format!("--enable-{}={}", self.key, val.as_str()),
            (true, Value::Bool(false)) => format!("--without-{}", self.key),
            (true, val) => format!("--with-{}={}", self.key, val.as_str()),
        }
    }
}

pub enum Value<'a> {
    Bool(bool),
    Str(&'a str),
}

impl<'a> Value<'a> {
    pub const fn yes() -> Self {
        Self::Bool(true)
    }

    pub const fn no() -> Self {
        Self::Bool(true)
    }

    pub const fn str(string: &'a str) -> Self {
        Self::Str(string)
    }

    pub const fn as_str(&self) -> &str {
        match self {
            Value::Bool(true) => "true",
            Value::Bool(false) => "false",
            Value::Str(string) => string,
        }
    }
}

impl<'a> From<bool> for Value<'a> {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl<'a> From<&'a str> for Value<'a> {
    fn from(value: &'a str) -> Self {
        Self::Str(value)
    }
}

impl<'a> AsRef<str> for Value<'a> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> AsRef<OsStr> for Value<'a> {
    fn as_ref(&self) -> &OsStr {
        self.as_str().as_ref()
    }
}

pub const fn autotools<'a>() -> Autotools<'a> {
    Autotools {
        definitions: &[],
        inclusions: &[],
    }
}

impl<'a> Autotools<'a> {
    pub fn definitions(&mut self, definitions: &'a [Pair<'a>]) -> &mut Self {
        self.definitions = definitions;
        self
    }

    pub fn inclusions(&mut self, inclusions: &'a [Pair<'a>]) -> &mut Self {
        self.inclusions = inclusions;
        self
    }

    pub fn get_definitions(&self) -> impl Iterator<Item = String> + '_ {
        self.definitions.iter().map(|pair| pair.to_string(false))
    }

    pub fn get_inclusions(&self) -> impl Iterator<Item = String> + '_ {
        self.inclusions.iter().map(|pair| pair.to_string(true))
    }

    pub async fn execute(&mut self, build: &Build) -> crate::Result<()> {
        let mut command = Command::new("./configure");

        command
            .arg(format!("--prefix={}", build.install_dir().display()))
            .args(self.get_definitions())
            .args(self.get_inclusions())
            .current_dir(build.source_dir().as_path())
            .env_clear()
            .env("PATH", "/bin")
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());

        command.print(build.config(), "configure").await?;
        command.fancy_spawn().await?;

        super::make().execute(build).await?;
        super::make().subcommand("install").execute(build).await?;

        Ok(())
    }
}

use self::lexer::Lexer;
use self::parser::{Check, Parser, Status};

async fn process_line(build: &Build, line: &str) -> crate::Result<()> {
    let line = strip_ansi_escapes::strip(&line)?;
    let line = String::from_utf8_lossy(&line);
    let tokens: Vec<_> = Lexer::new(&line).collect();
    let mut parser = Parser::new(&line);
    let status = parser.parse();

    match status {
        Status::Check(Check::Any(ident, status)) => {
            let mut line = Line::new(" ->", Colour::None);
            let display = &ident;

            line.append("check", Colour::Green)
                .append(format!("\"{display}\""), Colour::Blue)
                .append("exists?", Colour::None);

            append_status(&mut line, status);

            line.newline().render(build.config().shell()).await?;
        }
        Status::Check(Check::Header(path, status)) => {
            let mut line = Line::new(" ->", Colour::None);
            let display = path.display();

            line.append("check", Colour::Green)
                .append("header", Colour::None)
                .append(format!("<{display}>"), Colour::Green)
                .append("exists?", Colour::None);

            append_status(&mut line, status);

            line.newline().render(build.config().shell()).await?;
        }
        Status::Check(Check::Type(ident, status)) => {
            let mut line = Line::new(" ->", Colour::None);
            let display = &ident;

            line.append("check", Colour::Green)
                .append("type of", Colour::None)
                .append(format!("\"{display}\""), Colour::Blue)
                .raw_append("?", Colour::None);

            append_status(&mut line, status);

            line.newline().render(build.config().shell()).await?;
        }
        Status::Skip => {}
        _ => {
            Line::new(" !!", Colour::None)
                .append("debug", Colour::Blue)
                .append(format!("{tokens:?}"), Colour::None)
                .newline()
                .append("!!", Colour::None)
                .append("debug", Colour::Blue)
                .append(format!("{status:?}"), Colour::None)
                .newline()
                .render(build.config().shell())
                .await?;
        }
    }

    /*
    match &parts[..] {
        ["creating", file] | [_, "creating", file] => {
            let start = unsafe { file.as_ptr().offset_from(lower.as_ptr()) as usize };
            let file = Path::new(&line[start..start.saturating_add(file.len())]);
            let file = file.display();

            Line::new(" ->", Colour::None)
                .append("generate", Colour::Green)
                .append(format!("\"{file}\""), Colour::Magenta)
                .newline()
                .render(build.config().shell())
                .await?;
        }
        ["checking", _what, "usability...", ..] | ["checking", _what, "presence...", ..] => {}
        all @ ["checking", "for", what, rest @ ..] => {
            let what = what.trim_end_matches("...");
            let header = what.contains(".h");
            let status = match rest {
                ["(cached)", status] => status,
                [status] => status,
                _ => {
                    Line::new(" !!", Colour::None)
                        .append("debug", Colour::Blue)
                        .append(format!("{all:?}"), Colour::None)
                        .newline()
                        .render(build.config().shell())
                        .await?;

                    return Ok(());
                }
            };

            let mut line = Line::new(" ->", Colour::None);

            line.append("check", Colour::Green);

            if header {
                line.append("header", Colour::None)
                    .append(format!("<{what}>"), Colour::Green)
                    .append("exists?", Colour::None);
            } else {
                line.append("for", Colour::None).append(what, Colour::Blue);
            }

            if status.contains("yes") {
                line.append("yes", Colour::Green);
            } else {
                line.append("no", Colour::Red);
            }

            line.newline().render(build.config().shell()).await?;
        }
        ["checking", kind @ "build", _system, _type, target, rest @ ..]
        | ["checking", kind @ "host", _system, _type, target, rest @ ..]
        | ["checking", kind @ "target", _system, _type, target, rest @ ..] => {
            Line::new(" ->", Colour::None)
                .append("check", Colour::Green)
                .append(kind, Colour::None)
                .append("system", Colour::None)
                .append(target, Colour::Blue)
                .newline()
                .render(build.config().shell())
                .await?;
        }
        ["checking", "if", "you", "want", rest @ ..] => {
            Line::new(" ->", Colour::None)
                .append("check", Colour::Green)
                .append(format!("{rest:?}"), Colour::None)
                .newline()
                .render(build.config().shell())
                .await?;
        }
        ["checking", "if", rest @ ..] => {
            Line::new(" ->", Colour::None)
                .append("check", Colour::Green)
                .append(format!("{rest:?}"), Colour::None)
                .newline()
                .render(build.config().shell())
                .await?;
        }
        ["checking", "for", rest @ ..] => {
            Line::new(" ->", Colour::None)
                .append("check", Colour::Green)
                .append(format!("{rest:?}"), Colour::None)
                .newline()
                .render(build.config().shell())
                .await?;
        }
        rest => {
            Line::new(" !!", Colour::None)
                .append("debug", Colour::Blue)
                .append(format!("{rest:?}"), Colour::None)
                .newline()
                .render(build.config().shell())
                .await?;
        }
    }*/

    Ok(())
}

fn append_status(line: &mut Line, status: Option<bool>) {
    match status {
        Some(true) => {
            line.append("yes", Colour::Green);
        }
        Some(false) => {
            line.append("no", Colour::Red);
        }
        _ => {}
    }
}
