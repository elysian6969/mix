use super::process::{Command, Stdio};
use crate::ops::install::build::Build;
use std::ffi::OsStr;
use std::marker::PhantomData;
use std::path::Path;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};

pub struct Meson<'a> {
    args: &'a [&'a str],
    subcommand: Subcommand,
    phantom: PhantomData<&'a ()>,
}

pub enum Subcommand {
    Setup,
    Configure,
    Dist,
    Install,
    Introspect,
    Init,
    Test,
    Wrap,
    Subprojects,
    Help,
    Rewrite,
    Compile,
}

impl Subcommand {
    pub const fn as_str(&self) -> &'static str {
        use Subcommand::*;

        match self {
            Setup => "setup",
            Configure => "configure",
            Dist => "dist",
            Install => "install",
            Introspect => "introspect",
            Init => "init",
            Test => "test",
            Wrap => "wrap",
            Subprojects => "subprojects",
            Help => "help",
            Rewrite => "rewrite",
            Compile => "compile",
        }
    }
}

impl AsRef<str> for Subcommand {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<OsStr> for Subcommand {
    fn as_ref(&self) -> &OsStr {
        self.as_str().as_ref()
    }
}

impl<'a> Meson<'a> {
    pub fn args(&mut self, args: &'a [&'a str]) -> &mut Self {
        self.args = args;
        self
    }

    pub fn subcommand(&mut self, subcommand: Subcommand) -> &mut Self {
        self.subcommand = subcommand;
        self
    }

    pub async fn execute(&mut self, build: &Build) -> crate::Result<()> {
        let builderer = build.build_dir().join("build");

        fs::create_dir_all(&builderer).await?;

        let mut command = Command::new("meson");

        command
            .arg(&self.subcommand)
            .args(self.args.iter())
            .arg(build.source_dir().as_path())
            .current_dir(&builderer)
            .env_clear()
            .env("PATH", "/bin")
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());

        command.print(build.config(), "meson").await?;
        command.fancy_spawn().await?;

        Ok(())
    }
}

/// create a new meson invocation
pub const fn meson<'a>() -> Meson<'a> {
    Meson {
        args: &[],
        subcommand: Subcommand::Setup,
        phantom: PhantomData,
    }
}
