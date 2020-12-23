use std::{
    fs,
    path::{Path, PathBuf},
};
use tokio::process::Command;

pub struct Git {
    branch: Option<String>,
    dest: PathBuf,
    inner: Command,
}

impl Git {
    pub fn clone<R, D>(repo: R, dest: D) -> Self
    where
        R: AsRef<str>,
        D: AsRef<Path>,
    {
        let dest = dest.as_ref().to_path_buf();
        let mut inner = Command::new("git");

        inner
            .arg("clone")
            .arg("--depth=1")
            .arg("--recursive")
            .arg(repo.as_ref())
            .arg(&dest)
            .current_dir(&dest)
            .env_clear();

        Self {
            branch: None,
            dest,
            inner,
        }
    }

    pub fn branch<B: AsRef<str>>(&mut self, branch: B) -> &mut Self {
        self.branch = Some(branch.as_ref().to_string());
        self
    }

    pub async fn execute(&mut self) -> anyhow::Result<()> {
        fs::create_dir_all(&self.dest)?;

        let inner = &mut self.inner;

        if let Some(branch) = self.branch.as_ref() {
            inner.arg(format!("--branch={}", branch));
        }

        let mut child = self.inner.spawn()?;

        child.wait().await?;

        Ok(())
    }
}
