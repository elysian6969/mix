use super::process::{Command, Stdio};
use crate::config::Config;
use crate::shell::Text;
use crossterm::style::Colorize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, BufReader};
use ufmt::derive::uDebug;

pub use self::error::{Error, Exit};

mod error;

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

    /// --enable/--disable
    defines: HashMap<String, Value>,

    /// --with/--without
    includes: HashMap<String, Value>,
}

impl Autotools {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            defines: HashMap::new(),
            includes: HashMap::new(),
        }
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

    pub async fn execute(&mut self, config: &Config) -> crate::Result<()> {
        let mut command = Command::new("./configure");

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

        command.print(config, "config").await?;

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
            let mut parts = line.split_whitespace();

            match (parts.next(), parts.next(), parts.next()) {
                (Some("checking"), Some("for"), Some(what)) => {
                    let what = what.trim_end_matches("...");
                    let header = what.contains(".h");

                    if header {
                        println!(
                            " -> {action} for header {what}",
                            action = "check".green(),
                            what = format!("<{what}>").green()
                        );
                    } else {
                        println!(
                            " -> {action} for {what}",
                            action = "check".green(),
                            what = what.blue()
                        );
                    }
                }
                debug => {
                    let rest: Vec<_> = parts.collect();

                    //println!(" !! debug {debug:?} {rest:?}");
                }
            }

            /*
            if let Some(action) = parts.next() {
                if let Some(action2) = parts.next() {
                    let rest: Vec<_> = parts.collect();

                    println!(" -> {action:?} {action2:?} {rest:?}");
                } else {
                    let rest: Vec<_> = parts.collect();

                    println!(" -> {action:?} {rest:?}");
                }
            }*/
        }

        stderr_handle.await?;
        wait_handle.await?;

        let mut command = Command::new("make");

        command
            .arg("-j1")
            .current_dir(&self.path)
            .env_clear()
            .env("PATH", "/bin")
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());

        command.print(config, "build").await?;

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

        stderr_handle.await?;
        wait_handle.await?;

        Ok(())
    }
}
