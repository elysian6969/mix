use super::process::{Command, Stdio};
use crate::ops::install::build::Build;
use std::path::Path;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};

pub struct Cmake<'a> {
    build_dir: &'a Path,
    prefix: &'a Path,
    jobs: usize,
}

impl<'a> Cmake<'a> {
    pub fn prefix(&mut self, prefix: &'a Path) -> &mut Self {
        self.prefix = prefix;
        self
    }

    pub async fn execute(&mut self, build: &Build) -> crate::Result<()> {
        let mut command = Command::new("cmake");

        let builderer = build.build_dir().join("build");

        fs::create_dir_all(&builderer).await?;

        command
            .arg("-S")
            .arg(self.build_dir)
            .arg("-B")
            .arg(&builderer)
            .arg(format!("-DCMAKE_INSTALL_PREFIX={}", self.prefix.display()))
            .current_dir(&builderer)
            .env_clear()
            .env("PATH", "/bin")
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());

        command.print(build.config(), "cmake").await?;

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

        let mut command = Command::new("cmake");

        command
            .arg("--build")
            .arg(&builderer)
            .current_dir(&builderer)
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

        let mut command = Command::new("cmake");

        command
            .arg("--install")
            .arg(&builderer)
            .current_dir(&builderer)
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

/// create a new cmake invocation
pub fn cmake<'a>(build_dir: &'a Path) -> Cmake<'a> {
    Cmake {
        build_dir: build_dir,
        prefix: Path::new("/usr/local"),
        jobs: 1,
    }
}
