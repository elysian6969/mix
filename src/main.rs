use crossterm::style;
use crossterm::style::{Colorize, Styler};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::iter;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use tokio::process::Command;

pub mod shell {
    use crossterm::style::{Colorize, Styler};
    use std::fmt;

    /// Action to print to the terminal
    #[derive(Clone, Copy, Debug)]
    pub enum Action {
        Compiling,
        Configuring,
        Installing,
        Updating,
    }

    impl Action {
        pub fn as_str(&self) -> &str {
            match self {
                Action::Compiling => "compiling",
                Action::Configuring => "configuring",
                Action::Installing => "installing",
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
        pub fn ias_str(&self) -> &str {
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

pub mod spec {
    use super::candy::{Candy, Dirs};
    use super::shell::{Action, Status};
    use super::triple::Triple;
    // FIXME: use nix::{pty, unistd};
    use fs_extra::dir::CopyOptions;
    use serde::{Deserialize, Serialize};
    use std::fs;
    use std::os::unix::io::FromRawFd;
    use std::process::Stdio;
    use tokio::process::Command;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Package<'package> {
        name: &'package str,
        version: &'package str,
        source: &'package str,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Configure<'config> {
        #[serde(borrow)]
        disable: Option<Vec<&'config str>>,
        #[serde(borrow)]
        enable: Option<Vec<&'config str>>,
        #[serde(borrow)]
        with: Option<Vec<&'config str>>,
        #[serde(borrow)]
        without: Option<Vec<&'config str>>,
    }

    impl<'configure> Configure<'configure> {
        pub async fn configure<'spec>(
            &self,
            spec: &Spec<'spec>,
            dirs: &Dirs,
        ) -> anyhow::Result<()> {
            println!("{} {}", Action::Configuring, spec.name());

            let mut args = vec![
                format!("--prefix={}", dirs.target.display()),
                format!("--bindir={}/bin", dirs.target.display()),
                format!("--sbindir={}/bin", dirs.target.display()),
                format!("--libexecdir={}/bin", dirs.target.display()),
                format!("--sysconfdir={}/conf", dirs.target.display()),
                format!("--sharedstatedir={}/conf", dirs.target.display()),
                format!("--localstatedir={}/conf", dirs.target.display()),
                format!("--libdir={}/lib", dirs.target.display()),
                format!("--includedir={}/abi", dirs.target.display()),
                format!("--oldincludedir={}/abi", dirs.target.display()),
                format!("--datarootdir={}/conf", dirs.target.display()),
                format!("--datadir={}/conf", dirs.target.display()),
                format!("--infodir={}/info", dirs.target.display()),
                format!("--mandir={}/info/man", dirs.target.display()),
                format!("--localedir={}/info/locale", dirs.target.display()),
                format!("--docdir={}/info", dirs.target.display()),
            ];

            if let Some(ref opts) = self.disable {
                args.extend(opts.iter().map(|opt| format!("--disable-{}", opt)));
            }

            if let Some(ref opts) = self.enable {
                args.extend(opts.iter().map(|opt| format!("--enable-{}", opt)));
            }

            if let Some(ref opts) = self.with {
                args.extend(opts.iter().map(|opt| format!("--with-{}", opt)));
            }

            if let Some(ref opts) = self.without {
                args.extend(opts.iter().map(|opt| format!("--without-{}", opt)));
            }

            if dirs.build.exists() {
                fs::remove_dir_all(&dirs.build)?;
            }

            fs::create_dir_all(&dirs.build)?;

            //let pair = pty::openpty(None, None)?;
            //let stderr = unistd::dup(pair.slave)?;

            let configure = dirs.source.clone().join("configure");

            if !configure.exists() {
                println!("{} package does not have a configure script", Status::Error);
            }

            let mut child = Command::new(configure)
                .args(&args)
                .current_dir(&dirs.build)
                .env("CC", "/usr/bin/gcc")
                .env("CXX", "/usr/bin/g++")
                .kill_on_drop(true)
                //     .stderr(unsafe { Stdio::from_raw_fd(stderr) })
                //     .stdout(unsafe { Stdio::from_raw_fd(pair.slave) })
                .spawn()?;

            let status = child.wait().await?;

            println!("{} {}", Action::Compiling, spec.name());

            //let pair = pty::openpty(None, None)?;

            let mut child = Command::new("make")
                .current_dir(&dirs.build)
                .kill_on_drop(true)
                //    .stderr(unsafe { Stdio::from_raw_fd(pair.slave) })
                //    .stdout(unsafe { Stdio::from_raw_fd(pair.slave) })
                .spawn()?;

            let status = child.wait().await?;

            println!("{} {}", Action::Installing, spec.name());

            //let pair = pty::openpty(None, None)?;

            fs::remove_dir_all(&dirs.target)?;
            fs::create_dir_all(&dirs.target)?;

            let mut child = Command::new("make")
                .arg("install")
                .current_dir(&dirs.build)
                .kill_on_drop(true)
                //    .stderr(unsafe { Stdio::from_raw_fd(pair.slave) })
                //    .stdout(unsafe { Stdio::from_raw_fd(pair.slave) })
                .spawn()?;

            let status = child.wait().await?;

            let bin = dirs.target.clone().join("sbin");
            let sbin = dirs.target.clone().join("sbin");

            if sbin.exists() {
                println!(
                    "{} package failed to respect configuration, /sbin exists",
                    Status::Warning
                );

                let files: Vec<_> = fs::read_dir(&sbin)?
                    .flatten()
                    .map(|entry| entry.path())
                    .collect();

                fs_extra::move_items(
                    &files,
                    bin,
                    &CopyOptions {
                        overwrite: false,
                        skip_exist: true,
                        buffer_size: 64000,
                        copy_inside: true,
                        content_only: false,
                        depth: 0,
                    },
                )?;
            }

            fs::remove_dir_all(&dirs.build)?;

            Ok(())
        }
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Spec<'spec> {
        #[serde(borrow)]
        package: Package<'spec>,
        #[serde(borrow)]
        configure: Option<Configure<'spec>>,
    }

    impl<'spec> Spec<'spec> {
        pub fn name(&self) -> &'spec str {
            &self.package.name
        }

        pub fn github_url(&self) -> String {
            format!("https://github.com/{}", self.package.source)
        }

        pub async fn build<'build>(
            &self,
            candy: &Candy,
            triple: &Triple<'build>,
        ) -> anyhow::Result<()> {
            let dirs = candy.dirs_of(&self, &triple);
            let github_url = self.github_url();

            println!("{} {}", Action::Updating, self.name());

            //let pair = pty::openpty(None, None)?;

            let mut child = Command::new("git")
                .args(&["clone", "--depth=1"])
                .arg(&github_url)
                .arg(&dirs.source)
                .kill_on_drop(true)
                //    .stderr(unsafe { Stdio::from_raw_fd(pair.slave) })
                //    .stdout(unsafe { Stdio::from_raw_fd(pair.slave) })
                .spawn()?;

            let status = child.wait().await?;

            match &self.configure {
                Some(configure) => configure.configure(&self, &dirs).await,
                _ => Ok(()),
            }
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
        pub const fn linux(mut self) -> Self {
            self.sys("linux")
        }

        /// Set the ABI to GNU
        pub const fn gnu(mut self) -> Self {
            self.abi("gnu")
        }

        /// Set the ABI to musl
        pub const fn musl(mut self) -> Self {
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
            self.root().join("source").join(spec.name())
        }

        /// Return the build directory of a spec relative to this
        /// instance's root directory
        ///
        /// Panics with invalid target triples
        pub fn build_of(&self, spec: &Spec, triple: &Triple) -> PathBuf {
            self.root()
                .join("build")
                .join(triple.to_string().unwrap())
                .join(spec.name())
        }

        /// Return the target directory of a spec relative to this
        /// instance's root directory
        ///
        /// Panics with invalid target triples
        pub fn target_of(&self, spec: &Spec, triple: &Triple) -> PathBuf {
            self.root()
                .join(triple.to_string().unwrap())
                .join(spec.name())
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

    let mut file = File::open(&cmdline.spec)?;
    let meta = file.metadata()?;
    let mut buffy = vec![0; meta.len() as usize];

    file.read(&mut buffy)?;

    let candy = candy::Candy::new(&"/milk");
    let spec: spec::Spec<'_> = toml::de::from_slice(&buffy)?;

    spec.build(&candy, &triple::X86_64_UNKNOWN_LINUX_GNU)
        .await?;

    Ok(())
}
