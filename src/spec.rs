use super::shell::{Action, Status};
use super::triple::Triple;
use super::util::Git;
use super::watson::{Dirs, Watson};
use fs_extra::dir::CopyOptions;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use tokio::process::Command;

/// defines name, version, sources
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub source: String,
    pub branch: Option<String>,
}

/// list of actions to execute in a stage
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Actions(Option<Vec<String>>);

impl Actions {
    pub async fn execute(&self, action: Action, spec: &Spec, dirs: &Dirs) -> anyhow::Result<()> {
        if self.0.is_some() {
            println!("{} {}", action, &spec.package.name);
        }

        for action in self.0.iter().flatten() {
            let action = action
                .replace("%build", &dirs.build.display().to_string())
                .replace("%prefix", &dirs.target.display().to_string())
                .replace("%source", &dirs.source.display().to_string());

            let args = shell_words::split(&action)?;

            if args.len() == 0 {
                continue;
            }

            println!("{} {} {:?}", Action::Running, spec.package.name, &args);

            if args[0].starts_with("@") {
                match args[0].as_str() {
                    "@copy" => {
                        let src = Path::new(&args[1]);
                        let dst = Path::new(&args[2]);

                        if !src.exists() {
                            println!(
                                "{} {} {} {}",
                                Action::Running,
                                spec.package.name,
                                Status::Warning,
                                "@copy: <source> parameter doesn't exist"
                            );
                        }

                        if !dst.exists() {
                            println!(
                                "{} {} {} {}",
                                Action::Running,
                                spec.package.name,
                                Status::Warning,
                                "@copy: <destination> parameter doesn't exist"
                            );
                        }

                        fs_extra::copy_items(&[&args[1]], &args[2], &CopyOptions::default())?;
                    }
                    "@replicate" => {
                        let entries: Vec<_> = fs::read_dir(&dirs.source)?
                            .flatten()
                            .map(|entry| entry.path())
                            .collect();

                        fs_extra::copy_items(&entries, &dirs.build, &CopyOptions::default())?;
                    }
                    _ => println!(
                        "{} {} {} {}",
                        Action::Running,
                        spec.package.name,
                        Status::Warning,
                        "unknown builtin function"
                    ),
                }
            } else {
                let mut child = Command::new(&args[0])
                    .args(&args[1..])
                    .current_dir(&dirs.build)
                    .spawn()?;

                child.wait().await?;
            }
        }

        Ok(())
    }
}

// package spec
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Spec {
    pub package: Package,
    #[serde(default)]
    pub prepare: Actions,
    #[serde(default)]
    pub build: Actions,
    #[serde(default)]
    pub install: Actions,
}

impl Spec {
    pub async fn execute(&self, candy: &Watson, triple: &Triple<'_>) -> anyhow::Result<()> {
        let dirs = candy.dirs_of(&self, &triple);
        let source = format!("https://github.com/{}", &self.package.source);

        println!("{} {}", Action::Updating, &self.package.name);

        let mut git = Git::clone(source, &dirs.source);

        if let Some(branch) = self.package.branch.as_ref() {
            git.branch(branch);
        }

        git.execute().await?;

        if dirs.build.exists() {
            fs::remove_dir_all(&dirs.build)?;
        }

        fs::create_dir_all(&dirs.build)?;

        self.prepare
            .execute(Action::Preparing, &self, &dirs)
            .await?;

        self.build.execute(Action::Building, &self, &dirs).await?;

        if dirs.target.exists() {
            fs::remove_dir_all(&dirs.target)?;
        }

        fs::create_dir_all(&dirs.target)?;

        self.install
            .execute(Action::Installing, &self, &dirs)
            .await?;

        if dirs.build.exists() {
            fs::remove_dir_all(&dirs.build)?;
        }

        Ok(())
    }
}
