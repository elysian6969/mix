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

pub mod parser {
    use super::lexer::{Lexer, Token};
    use std::path::{Path, PathBuf};

    pub struct Parser<'a> {
        lexer: Lexer<'a>,
        lookahead: Option<Token<'a>>,
    }

    impl<'a> Parser<'a> {
        pub fn new(input: &'a str) -> Self {
            let mut lexer = Lexer::new(input);
            let lookahead = lexer.next();

            Self { lexer, lookahead }
        }

        fn step(&mut self) {
            self.lookahead = self.lexer.next();
        }

        fn current(&self) -> &Option<Token<'a>> {
            &self.lookahead
        }

        /// optionally consume a check
        fn consume_check(&mut self) -> Option<()> {
            match self.current() {
                Some(Token::Check) => Some(self.step()),
                _ => None,
            }
        }

        /// optionally consume an cached
        fn consume_cached(&mut self) -> Option<()> {
            match self.current() {
                Some(Token::Cached) => Some(self.step()),
                _ => None,
            }
        }

        /// optionally consume an elipsis
        fn consume_elipsis(&mut self) -> Option<()> {
            match self.current() {
                Some(Token::Elipsis) => Some(self.step()),
                _ => None,
            }
        }

        /// optionally consume a for
        fn consume_for(&mut self) -> Option<()> {
            match self.current() {
                Some(Token::For) => Some(self.step()),
                _ => None,
            }
        }

        /// optionally consume header
        fn consume_header(&mut self) -> Option<PathBuf> {
            match self.current() {
                Some(Token::Header(header)) => {
                    let result = Some(header.to_path_buf());

                    self.step();

                    result
                }
                _ => None,
            }
        }

        /// optionally consume ident
        fn consume_ident(&mut self) -> Option<String> {
            match self.current() {
                Some(Token::Ident(ident)) => {
                    let result = Some(ident.to_string());

                    self.step();

                    result
                }
                _ => None,
            }
        }

        /// optionally consume presence
        fn consume_presence(&mut self) -> Option<()> {
            match self.current() {
                Some(Token::Presence) => Some(self.step()),
                _ => None,
            }
        }

        /// optionally consume a type
        fn consume_type(&mut self) -> Option<()> {
            match self.current() {
                Some(Token::Type) => Some(self.step()),
                _ => None,
            }
        }

        /// optionally consume a status
        fn consume_status(&mut self) -> Option<bool> {
            let result = match self.current() {
                Some(Token::Yes) => Some(true),
                Some(Token::No) => Some(false),
                _ => None,
            };

            if result.is_some() {
                self.step();
            }

            result
        }

        /// optionally consume usability
        fn consume_usability(&mut self) -> Option<()> {
            match self.current() {
                Some(Token::Usability) => Some(self.step()),
                _ => None,
            }
        }

        /// optionally consume the rest of a check pattern
        fn consume_check_rest(&mut self) -> Option<bool> {
            let _ = self.consume_elipsis();
            let _ = self.consume_cached();

            self.consume_status()
        }

        /// optionally consume type pattern
        fn consume_check_type(&mut self) -> Option<String> {
            self.consume_type()?;
            self.consume_ident()
        }

        /// optionally consume presence or usability
        fn comsume_ignore(&mut self) -> Option<()> {
            self.consume_presence().or(self.consume_usability())
        }

        /// optionally consume check patterns
        fn consume_check_pattern(&mut self) -> Option<Status> {
            self.consume_check()?;
            self.consume_for()?;

            /*let pattern = self
            .consume_header()
            .map(Pattern::Header)
            .or_else(|| self.consume_ident().map(Pattern::Ident))
            .or_else(|| self.consume_check_type().map(Pattern::Type))?;*/

            self.step();
            dbg!(self.current());

            return Some(Status::Check(Check::None));

            /*let ignore = self.comsume_ignore().is_some();

            if ignore {
                return Some(Status::Check(Check::None));
            }

            let status = self.consume_check_rest();
            let check = match pattern {
                Pattern::Header(header) => Check::Header(header, status),
                Pattern::Ident(ident) => Check::Any(ident, status),
                Pattern::Type(ident) => Check::Type(ident, status),
            };

            Some(Status::Check(check))*/
        }

        pub fn parse(&mut self) -> Status {
            match self.consume_check_pattern() {
                Some(status) => status,
                _ => Status::None,
            }
        }
    }

    #[derive(Debug)]
    pub enum Status {
        Check(Check),
        None,
    }

    #[derive(Debug)]
    pub enum Check {
        Any(String, Option<bool>),
        Header(PathBuf, Option<bool>),
        Type(String, Option<bool>),
        None,
    }

    #[derive(Debug)]
    enum Pattern {
        Header(PathBuf),
        Ident(String),
        Type(String),
    }
}

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

        /*
        let mut command = Command::new("make");

        command
            .arg("-j18")
            .current_dir(& self.path)
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

                //println!("stderr: {parts:?}");
            }

            Ok::<_, crate::Error>(())
        });

        while let Some(line) = stdout.next_line().await? {
            let line = strip_ansi_escapes::strip(&line)?;
            let line = String::from_utf8_lossy(&line).to_lowercase();
            let mut parts: Vec<_> = line.split_whitespace().collect();

            //println!("stdout: {parts:?}");
        }

        stderr_handle.await??;
        wait_handle.await?;*/

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
        Status::Check(Check::None) => {}
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
