use super::process::{Command, Stdio};
use crate::ops::install::build::Build;
use std::path::Path;
use tokio::io::{AsyncBufReadExt, BufReader};

pub struct Make<'a> {
    subcommand: &'a str,
}

impl<'a> Make<'a> {
    pub fn subcommand(&mut self, subcommand: &'a str) -> &mut Self {
        self.subcommand = subcommand;
        self
    }

    pub async fn execute(&mut self, build: &Build) -> crate::Result<()> {
        let mut command = Command::new("make");

        command
            .arg(self.subcommand)
            .arg("--jobs")
            .arg(build.jobs().to_string())
            .current_dir(build.source_dir().as_path())
            .env_clear()
            .env("PATH", "/bin")
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());

        command.print(build.config(), "build").await?;
        command.fancy_spawn().await?;

        Ok(())
    }
}

/// create a new make invocation
pub const fn make<'a>() -> Make<'a> {
    Make { subcommand: "all" }
}
