use crate::config::Config;
use crate::shell::Text;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use tokio::process::Command;
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
        let mut command = StdCommand::new("./configure");

        command
            .args(self.get_includes())
            .args(self.get_defines())
            .current_dir(&self.path)
            .env_clear()
            .env("AR", "/bin/llvm-ar")
            .env("CC", "/bin/clang");

        //command.arg(format!("--prefix={}", config.prefix()));
        let mut program = command
            .get_program()
            .to_str()
            .expect("infallible")
            .to_string();

        let current_dir = command
            .get_current_dir()
            .expect("infallible")
            .display()
            .to_string();

        let args: Vec<&str> = command.get_args().flat_map(|arg| arg.to_str()).collect();

        program.push_str(&args.join(" "));

        let buffer =
            ufmt::uformat!(" -> executing {} in {}\n", program, current_dir).expect("infallible");

        Text::new(buffer).render(config.shell()).await?;

        let mut command = Command::from(command);
        let mut child = command.spawn()?;
        let status = child.wait().await?;

        if !status.success() {
            return Err(Box::new(match status.code() {
                Some(code) => Error::Exit(Exit::Code(code)),
                None => Error::Exit(Exit::Signal),
            }));
        }

        let mut command = StdCommand::new("make");

        command
            .arg("-j1")
            .current_dir(&self.path)
            .env_clear()
            .env("PATH", "/bin");

        let mut program = command
            .get_program()
            .to_str()
            .expect("infallible")
            .to_string();

        let current_dir = command
            .get_current_dir()
            .expect("infallible")
            .display()
            .to_string();

        let args: Vec<&str> = command.get_args().flat_map(|arg| arg.to_str()).collect();

        if !args.is_empty() {
            program.push(' ');
            program.push_str(&args.join(" "));
        }

        let buffer =
            ufmt::uformat!(" -> executing {} in {}\n", program, current_dir).expect("infallible");

        Text::new(buffer).render(config.shell()).await?;

        let mut command = Command::from(command);
        let mut child = command.spawn()?;
        let status = child.wait().await?;

        if !status.success() {
            return Err(Box::new(match status.code() {
                Some(code) => Error::Exit(Exit::Code(code)),
                None => Error::Exit(Exit::Signal),
            }));
        }

        Ok(())
    }
}
