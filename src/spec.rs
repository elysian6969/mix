use {
    super::{
        config::Config,
        delete_on_drop::DeleteOnDrop,
        shell::{Action, Status},
        triple::Triple,
        util::Git,
    },
    fs_extra::dir::CopyOptions,
    serde::{Deserialize, Serialize},
    std::{fs, path::Path},
    tokio::process::Command,
};

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

            println!("{} {} {:?}", Action::Build, spec.package.name, &args);

            if args[0].starts_with("@") {
                match args[0].as_str() {
                    "@copy" => {
                        let src = Path::new(&args[1]);
                        let dst = Path::new(&args[2]);

                        if !src.exists() {
                            println!(
                                "{} {} {} {}",
                                Action::Build,
                                spec.package.name,
                                Status::Warning,
                                "@copy: <source> parameter doesn't exist"
                            );
                        }

                        if !dst.exists() {
                            println!(
                                "{} {} {} {}",
                                Action::Build,
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
                        Action::Build,
                        spec.package.name,
                        Status::Warning,
                        "unknown builtin function"
                    ),
                }
            } else {
                let mut child = Command::new(&args[0])
                    .env("TERM", "linux")
                    .env("LANG", "C.UTF-8")
                    .args(&args[1..])
                    .current_dir(&dirs.build)
                    .spawn()?;

                child.wait().await?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Script {
    pub name: String,
    pub version: Version,
    pub source: String,
    pub configure: Option<Vec<String>>,
    pub make: Option<Vec<String>>,
}

pub async fn build(script: &Script, triple: &Triple, instance: &Instance) -> anyhow::Result<()> {
    //let build_dir = DeleteOnDrop::new();

    Ok(())
}
