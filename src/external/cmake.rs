use super::process::{Command, Stdio};
use crate::ops::install::build::Build;
use std::marker::PhantomData;
use std::path::Path;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};

pub struct Cmake<'a> {
    phantom: PhantomData<&'a ()>,
}

impl<'a> Cmake<'a> {
    pub async fn execute(&mut self, build: &Build) -> crate::Result<()> {
        let mut command = Command::new("cmake");

        let builderer = build.build_dir().join("build");

        fs::create_dir_all(&builderer).await?;

        command
            .arg("-S")
            .arg(build.source_dir().as_path())
            .arg("-B")
            .arg(&builderer)
            .arg(format!(
                "-DCMAKE_INSTALL_PREFIX={}",
                build.install_dir().display()
            ))
            .current_dir(&builderer)
            .env_clear()
            .env("CC", "gcc")
            .env("CXX", "g++")
            .env("PATH", "/bin")
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());

        command.print(build.config(), "cmake").await?;
        command.fancy_spawn().await?;

        let mut command = Command::new("cmake");

        command
            .arg("--build")
            .arg(&builderer)
            .arg("--parallel")
            .arg(build.jobs().to_string())
            .current_dir(&builderer)
            .env_clear()
            .env("PATH", "/bin")
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());

        command.print(build.config(), "cmake").await?;
        command.fancy_spawn().await?;

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

        command.print(build.config(), "cmake").await?;
        command.fancy_spawn().await?;

        Ok(())
    }
}

/// create a new cmake invocation
pub fn cmake<'a>() -> Cmake<'a> {
    Cmake {
        phantom: PhantomData,
    }
}
