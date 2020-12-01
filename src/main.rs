use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use structopt::StructOpt;

pub mod shell {
    use crossterm::style::{Colorize, Styler};
    use std::fmt;

    /// Action to print to the terminal
    #[derive(Clone, Copy, Debug)]
    pub enum Action {
        Building,
        Installing,
        Preparing,
        Running,
        Updating,
    }

    impl Action {
        pub fn as_str(&self) -> &str {
            match self {
                Action::Building => "building",
                Action::Installing => "installing",
                Action::Preparing => "preparing",
                Action::Running => "running",
                Action::Updating => "updating",
            }
        }

        pub fn to_display(&self) -> impl fmt::Display {
            format!("{: >12}", self.as_str().to_owned().green().bold())
        }
    }

    impl fmt::Display for Action {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.to_display())
        }
    }

    /// Action to print to the terminal
    #[derive(Clone, Copy, Debug)]
    pub enum Status {
        Error,
        Warning,
    }

    impl Status {
        pub fn as_str(&self) -> &str {
            match self {
                Status::Error => "error",
                Status::Warning => "warning",
            }
        }

        pub fn to_display(&self) -> impl fmt::Display {
            let text = match self {
                Status::Error => "error".red(),
                Status::Warning => "warning".yellow(),
            };

            format!("{}:", text)
        }
    }

    impl fmt::Display for Status {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.to_display())
        }
    }
}

pub mod util {
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
}

pub mod spec {
    use super::candy::{Candy, Dirs};
    use super::shell::{Action, Status};
    use super::triple::Triple;
    use super::util::Git;
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
        pub async fn execute(
            &self,
            action: Action,
            spec: &Spec,
            dirs: &Dirs,
        ) -> anyhow::Result<()> {
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
        pub async fn execute(&self, candy: &Candy, triple: &Triple<'_>) -> anyhow::Result<()> {
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

            Ok(())
        }
    }
}

#[derive(Debug, StructOpt)]
pub struct Candy {
    #[structopt(parse(from_os_str))]
    spec: PathBuf,
}

pub mod triple {
    #[derive(Clone, Copy, Debug)]
    pub struct Triple<'triple> {
        arch: &'triple str,
        vendor: Option<&'triple str>,
        sys: Option<&'triple str>,
        abi: Option<&'triple str>,
    }

    impl<'triple> Triple<'triple> {
        /// Create new target triple with the specified architecture
        pub const fn new(arch: &'triple str) -> Self {
            Self {
                arch,
                vendor: None,
                sys: None,
                abi: None,
            }
        }

        /// Specify the vendor
        pub const fn vendor(mut self, vendor: &'triple str) -> Self {
            self.vendor = Some(vendor);
            self
        }

        /// Specify the system
        pub const fn sys(mut self, sys: &'triple str) -> Self {
            self.sys = Some(sys);
            self
        }

        /// Specify the ABI
        pub const fn abi(mut self, abi: &'triple str) -> Self {
            self.abi = Some(abi);
            self
        }

        /// Set the system to linux
        pub const fn linux(self) -> Self {
            self.sys("linux")
        }

        /// Set the ABI to GNU
        pub const fn gnu(self) -> Self {
            self.abi("gnu")
        }

        /// Set the ABI to musl
        pub const fn musl(self) -> Self {
            self.abi("musl")
        }

        pub fn to_string(&self) -> Option<String> {
            match &self {
                Triple {
                    arch,
                    vendor: Some(vendor),
                    sys: Some(sys),
                    abi: Some(abi),
                } => Some(format!("{}-{}-{}-{}", arch, vendor, sys, abi)),
                Triple {
                    arch,
                    vendor: None,
                    sys: Some(sys),
                    abi: Some(abi),
                } => Some(format!("{}-unknown-{}-{}", arch, sys, abi)),
                _ => None,
            }
        }
    }

    /// aarch64-unknown-linux-gnu
    pub const AARCH64_UNKNOWN_LINUX_GNU: Triple<'static> = Triple::new("aarch64").linux().gnu();
    /// aarch64-unknown-linux-musl
    pub const AARCH64_UNKNOWN_LINUX_MUSL: Triple<'static> = Triple::new("aarch64").linux().musl();

    /// x86_64-unknown-linux-gnu
    pub const X86_64_UNKNOWN_LINUX_GNU: Triple<'static> = Triple::new("x86_64").linux().gnu();
    /// x86_64-unknown-linux-musl
    pub const X86_64_UNKNOWN_LINUX_MUSL: Triple<'static> = Triple::new("x86_64").linux().musl();
}

pub mod candy {
    use super::spec::Spec;
    use super::triple::Triple;
    use std::path::{Path, PathBuf};

    /// This instance of Candy, lol
    #[derive(Debug)]
    pub struct Candy {
        root: PathBuf,
    }

    impl Candy {
        /// New instance with root
        pub fn new(root: &impl AsRef<Path>) -> Self {
            Self {
                root: root.as_ref().to_path_buf(),
            }
        }

        /// Return the root directory of this instance
        pub fn root(&self) -> PathBuf {
            self.root.clone()
        }

        /// Return the source directory of a spec relative to
        /// this instance's root directory
        pub fn source_of(&self, spec: &Spec) -> PathBuf {
            self.root().join("source").join(&spec.package.name)
        }

        /// Return the build directory of a spec relative to this
        /// instance's root directory
        ///
        /// Panics with invalid target triples
        pub fn build_of(&self, spec: &Spec, triple: &Triple) -> PathBuf {
            self.root()
                .join("build")
                .join(triple.to_string().unwrap())
                .join(&spec.package.name)
        }

        /// Return the target directory of a spec relative to this
        /// instance's root directory
        ///
        /// Panics with invalid target triples
        pub fn target_of(&self, spec: &Spec, triple: &Triple) -> PathBuf {
            self.root()
                .join(triple.to_string().unwrap())
                .join(&spec.package.name)
        }

        /// Shorthand for (source_of, build_of, target_of)
        pub fn dirs_of(&self, spec: &Spec, triple: &Triple) -> Dirs {
            Dirs {
                source: self.source_of(&spec),
                build: self.build_of(&spec, &triple),
                target: self.target_of(&spec, &triple),
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct Dirs {
        pub source: PathBuf,
        pub build: PathBuf,
        pub target: PathBuf,
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cmdline = Candy::from_args();

    let mut spec = File::open(&cmdline.spec)?;
    let metnya = spec.metadata()?;
    let mut buffy = vec![0; metnya.len() as usize];

    spec.read(&mut buffy)?;

    let candy = candy::Candy::new(&"/milk");
    let spec: spec::Spec = serde_yaml::from_slice(&buffy)?;

    spec.execute(&candy, &triple::X86_64_UNKNOWN_LINUX_GNU)
        .await?;

    Ok(())
}