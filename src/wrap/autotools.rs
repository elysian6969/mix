use std::collections::HashMap;
use std::path::PathBuf;
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
    /// --enable/--disable
    defines: HashMap<String, Value>,

    /// --with/--without
    includes: HashMap<String, Value>,
}

impl Autotools {
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

    pub async fn execute(&mut self) -> Result<(), Error> {
        let mut command = StdCommand::new("./configure");

        command.env_clear();
        //command.arg(format!("--prefix={}", config.prefix()));
        command.args(self.get_defines());
        command.args(self.get_includes());

        let args: Vec<&str> = command.get_args().flat_map(|arg| arg.to_str()).collect();
        let args = args.join(" ");

        let mut command = Command::from(command);
        let mut child = command.spawn()?;
        let status = child.wait().await?;

        if !status.success() {
            return Err(match status.code() {
                Some(code) => Error::Exit(Exit::Code(code)),
                None => Error::Exit(Exit::Signal),
            });
        }

        Ok(())
    }
}
