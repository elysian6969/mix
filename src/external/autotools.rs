use super::process::{Command, Stdio};
use crate::config::Config;
use crate::ops::install::build::Build;
use crate::shell::{Colour, Line, Text};
use crossterm::style::Colorize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, BufReader};
use ufmt::derive::uDebug;

pub use self::error::{Error, Exit};

mod error;

pub mod lexer;
pub mod parser;

#[derive(uDebug)]
pub enum Value {
    Bool(bool),
    String(String),
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

#[derive(uDebug)]
pub struct Autotools {
    /// path
    path: PathBuf,

    /// prefix
    prefix: Option<PathBuf>,

    /// --enable/--disable
    defines: HashMap<String, Value>,

    /// --with/--without
    includes: HashMap<String, Value>,
}

impl Autotools {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            prefix: None,
            defines: HashMap::new(),
            includes: HashMap::new(),
        }
    }

    pub fn prefix(&mut self, prefix: impl Into<PathBuf>) -> &mut Self {
        self.prefix = Some(prefix.into());
        self
    }

    pub fn define(&mut self, define: impl Into<String>, value: impl Into<Value>) -> &mut Self {
        self.defines.insert(define.into(), value.into());
        self
    }

    pub fn include(&mut self, define: impl Into<String>, value: impl Into<Value>) -> &mut Self {
        self.includes.insert(define.into(), value.into());
        self
    }

    pub fn get_defines(&self) -> impl Iterator<Item = String> + '_ {
        self.defines.iter().flat_map(|(define, value)| match value {
            Value::Bool(true) => ufmt::uformat!("--enable-{}", define),
            Value::Bool(false) => ufmt::uformat!("--disable-{}", define),
            Value::String(value) => ufmt::uformat!("--enable-{}={}", define, value),
        })
    }

    pub fn get_includes(&self) -> impl Iterator<Item = String> + '_ {
        self.includes
            .iter()
            .flat_map(|(include, value)| match value {
                Value::Bool(true) => ufmt::uformat!("--with-{}", include),
                Value::Bool(false) => ufmt::uformat!("--without-{}", include),
                Value::String(value) => ufmt::uformat!("--enable-{}={}", include, value),
            })
    }

    pub async fn execute(&mut self, build: &Build) -> crate::Result<()> {
        let mut command = Command::new("./configure");

        if let Some(prefix) = &self.prefix {
            let prefix = prefix.display();

            command.arg(format!("--prefix={prefix}"));
        }

        command
            .args(self.get_includes())
            .args(self.get_defines())
            .current_dir(&self.path)
            .env_clear()
            .env("AR", "llvm-ar")
            .env("CC", "clang")
            .env("PATH", "/bin")
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());

        command.print(build.config(), "config").await?;

        let mut child = command.spawn()?;

        let stderr = child
            .stderr
            .take()
            .expect("child did not have a handle to stderr");

        let stdout = child
            .stdout
            .take()
            .expect("child did not have a handle to stdout");

        let mut stderr = BufReader::new(stderr).lines();
        let mut stdout = BufReader::new(stdout).lines();

        let wait_handle = tokio::spawn(async move {
            // handle errors and status
            let _ = child.wait().await;
        });

        let stderr_handle = tokio::spawn(async move {
            while let Some(line) = stderr.next_line().await? {
                let line = strip_ansi_escapes::strip(&line)?;
                let line = String::from_utf8_lossy(&line).to_lowercase();
                let parts: Vec<_> = line.split_whitespace().collect();

                //println!("stderr: {parts:?}");
            }

            Ok::<_, crate::Error>(())
        });

        let reset = "\x1b[K\r";
        let newline = "\n";

        while let Some(line) = stdout.next_line().await? {
            process_line(build, &line).await?;
        }

        stderr_handle.await??;
        wait_handle.await?;

        let mut command = Command::new("make");

        command
            .arg("-j18")
            .current_dir(&self.path)
            .env_clear()
            .env("PATH", "/bin")
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());

        command.print(build.config(), "build").await?;

        let mut child = command.spawn()?;

        let stderr = child
            .stderr
            .take()
            .expect("child did not have a handle to stderr");

        let stdout = child
            .stdout
            .take()
            .expect("child did not have a handle to stdout");

        let mut stderr = BufReader::new(stderr).lines();
        let mut stdout = BufReader::new(stdout).lines();

        let wait_handle = tokio::spawn(async move {
            // handle errors and status
            let _ = child.wait().await;
        });

        let stderr_handle = tokio::spawn(async move {
            while let Some(line) = stderr.next_line().await? {
                let line = strip_ansi_escapes::strip(&line)?;
                let line = String::from_utf8_lossy(&line).to_lowercase();
                let mut parts: Vec<_> = line.split_whitespace().collect();

                println!("stderr: {parts:?}");
            }

            Ok::<_, crate::Error>(())
        });

        while let Some(line) = stdout.next_line().await? {
            let line = strip_ansi_escapes::strip(&line)?;
            let line = String::from_utf8_lossy(&line).to_lowercase();
            let mut parts: Vec<_> = line.split_whitespace().collect();

            println!("stdout: {parts:?}");
        }

        stderr_handle.await??;
        wait_handle.await?;

        let mut command = Command::new("make");

        command
            .arg("install")
            .arg("-j18")
            .current_dir(&self.path)
            .env_clear()
            .env("PATH", "/bin")
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());

        command.print(build.config(), "build").await?;

        let mut child = command.spawn()?;

        let stderr = child
            .stderr
            .take()
            .expect("child did not have a handle to stderr");

        let stdout = child
            .stdout
            .take()
            .expect("child did not have a handle to stdout");

        let mut stderr = BufReader::new(stderr).lines();
        let mut stdout = BufReader::new(stdout).lines();

        let wait_handle = tokio::spawn(async move {
            // handle errors and status
            let _ = child.wait().await;
        });

        let stderr_handle = tokio::spawn(async move {
            while let Some(line) = stderr.next_line().await? {
                let line = strip_ansi_escapes::strip(&line)?;
                let line = String::from_utf8_lossy(&line).to_lowercase();
                let mut parts: Vec<_> = line.split_whitespace().collect();

                println!("stderr: {parts:?}");
            }

            Ok::<_, crate::Error>(())
        });

        while let Some(line) = stdout.next_line().await? {
            let line = strip_ansi_escapes::strip(&line)?;
            let line = String::from_utf8_lossy(&line).to_lowercase();
            let mut parts: Vec<_> = line.split_whitespace().collect();

            println!("stdout: {parts:?}");
        }

        stderr_handle.await??;
        wait_handle.await?;

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
