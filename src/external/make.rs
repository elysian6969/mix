use super::process::{Command, Stdio};
use crate::config::Config;
use crate::ops::install::build::Build;
use crate::shell::{Colour, Line, Text};
use crossterm::style::Colorize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, BufReader};
use ufmt::derive::uDebug;

pub struct Make<'a> {
    build_dir: &'a Path,
    jobs: usize,
}

impl<'a> Make<'a> {
    pub async fn execute(&mut self, build: &Build) -> crate::Result<()> {
        let mut command = Command::new("make");

        command
            .arg("install")
            .arg("--jobs")
            .arg(self.jobs.to_string())
            .current_dir(&self.build_dir)
            .env_clear()
            .env("PATH", "/bin")
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());

        command.print(build.config(), "build").await?;

        let mut child = command.spawn()?;

        let stderr = unsafe { child.stderr.take().unwrap_unchecked() };
        let stdout = unsafe { child.stdout.take().unwrap_unchecked() };

        let mut stderr = BufReader::new(stderr).lines();
        let mut stdout = BufReader::new(stdout).lines();

        let wait = tokio::spawn(async move {
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
        wait.await?;

        Ok(())
    }
}

/// create a new make invocation
pub fn make<'a>(build_dir: &'a Path) -> Make<'a> {
    Make {
        build_dir: build_dir,
        jobs: 1,
    }
}
