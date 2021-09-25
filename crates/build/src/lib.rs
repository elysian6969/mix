#![feature(generators)]
#![feature(command_access)]
#![feature(format_args_capture)]

use crate::shell::Styles;
use command_extra::Line;
use futures_util::stream::StreamExt;
use milk_atom::Atom;
use milk_triple::Triple;
use path::{Path, PathBuf};
use std::env;
use std::ffi::{OsStr, OsString};
use std::process::Stdio;
use tokio::process::Command;
use tokio::runtime::Builder;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

mod autotools;

#[derive(Debug)]
pub enum Value {
    Bool(bool),
    String(String),
}

#[derive(Debug)]
pub struct Config {
    pub prefix: PathBuf,
    pub triple: Triple,
    pub atom: Atom,
    pub jobs: usize,
    pub define: Vec<(String, Value)>,
    pub include: Vec<(String, Value)>,
    pub build_dir: bool,
}

pub async fn build(config: Config) -> Result<()> {
    let repository_id = config.atom.repository_id.unwrap_or("core".try_into()?);
    let package_id = config.atom.package_id;
    let version = config.atom.version;

    let destination = config
        .prefix
        .join(config.triple.as_str())
        .join(repository_id.as_str())
        .join(package_id.as_str())
        .join(version.to_string());

    let current_dir: PathBuf = env::current_dir()?.into();
    let build_dir = if config.build_dir {
        let build_dir = current_dir.join("build");

        // TODO: Proper error handling,
        let _ = build_dir.create_dir_async();

        build_dir
    } else {
        current_dir.clone()
    };

    let styles = Styles::default();

    shell::header(&styles, "prefix", &config.prefix);
    shell::header(&styles, "triple", &config.triple);
    shell::header(&styles, "repository_id", &repository_id);
    shell::header(&styles, "package_id", &package_id);
    shell::header(&styles, "version", &version);
    shell::header(&styles, "destination", &destination);

    enum CargoAction {
        Update,
        Build,
    }

    pub struct Cargo {
        work_dir: PathBuf,
    }

    impl Cargo {
        pub fn new(work_dir: impl AsRef<Path>) -> Self {
            let work_dir = work_dir.as_ref().to_path_buf();

            Self { work_dir }
        }
    }

    if current_dir.join("Cargo.toml").exists_async().await {
        let mut command = Command::new("cargo");

        command
            .arg("build")
            .arg(format!("--jobs={}", config.jobs))
            .arg("--release")
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());

        let mut child = command.spawn()?;
        let stdio = command_extra::Stdio::from_child(&mut child)
            .ok_or("Failed to extract stdio from child.")?;
        let mut lines = stdio.lines();

        tokio::spawn(async move {
            // TODO: Proper error handling!
            let _ = child.wait().await;
        });

        while let Some(next) = lines.next().await {
            println!("{:?}", next);
        }
    } else {
        let autogen_file = current_dir.join("autogen.sh");

        if autogen_file.exists_async().await {
            let mut command = Command::new(&autogen_file);

            command
                .current_dir(&build_dir)
                .env_remove("CC")
                .env_remove("CFLAGS")
                .env_remove("CXX")
                .env_remove("CXXFLAGS")
                .env_remove("LIBS")
                .arg(format!("--prefix={}", &destination))
                .env("CC", "gcc")
                .env("CXX", "g++")
                .env("PREFIX", &destination)
                .stderr(Stdio::piped())
                .stdin(Stdio::null())
                .stdout(Stdio::piped());

            let mut child = command.spawn()?;
            let stdio = command_extra::Stdio::from_child(&mut child)
                .ok_or("Failed to extract stdio from child.")?;
            let mut lines = stdio.lines();

            tokio::spawn(async move {
                // TODO: Proper error handling!
                let _ = child.wait().await;
            });

            while let Some(line) = lines.next().await {
                match line? {
                    Line::Err(line) => shell::command_err(&styles, "autogen", line),
                    Line::Out(line) => shell::command_out(&styles, "autogen", line),
                }
            }
        }

        let configure_file = current_dir.join("configure");
        let mut command = Command::new(&configure_file);

        command
            .current_dir(&build_dir)
            .env_remove("CC")
            .env_remove("CFLAGS")
            .env_remove("CXX")
            .env_remove("CXXFLAGS")
            .env_remove("LIBS")
            .arg(format!("--prefix={}", &destination))
            .env("CC", "gcc")
            .env("CXX", "g++")
            .env("PREFIX", &destination)
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());

        /*let libc = Path::new("/milk/x86_64-linux-gnu/core/glibc/2.34.0");
        let libc_lib = libc.join("lib");
        let libc_include = libc.join("include");
        let dynamic_linker = libc_lib.join("ld-linux-x86-64.so.2");
        let crt1 = libc_lib.join("crt1.o");
        let crti = libc_lib.join("crti.o");
        let crtn = libc_lib.join("crtn.o");

        fn make_arg(flag: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> OsString {
            let mut arg: OsString = flag.as_ref().into();

            arg.push(value);
            arg
        }

        let mut cflags: Vec<OsString> = vec![];

        cflags.push("-fPIC".into());
        cflags.push("-nostdlib".into());
        cflags.push("-nostartfiles".into());
        //cflags.push("-static".into());
        cflags.push(make_arg("-I", &libc_include));
        cflags.push(make_arg("-L", &libc_lib));
        cflags.push(make_arg("-Wl,-dynamic-linker=", &dynamic_linker));
        cflags.push(make_arg("-Wl,-rpath=", &libc_lib));
        cflags.push(crt1.into());
        cflags.push(crti.into());
        cflags.push("/usr/lib/gcc/x86_64-pc-linux-gnu/10.2.0/crtbegin.o".into());
        cflags.push("-lc".into());
        cflags.push("-lgcc".into());
        cflags.push("/usr/lib/gcc/x86_64-pc-linux-gnu/10.2.0/crtend.o".into());
        cflags.push(crtn.into());
        cflags.push("-Wl,-no-pie".into());

        let cflags: Vec<_> = cflags.iter().flat_map(|s| s.to_str()).collect();
        let joined = cflags.join(" ");

        command.env("CFLAGS", &joined);*/

        if config.triple == Triple::i686() {
            command.env("CFLAGS", "-m32").env("CXXFLAGS:", "-m32");
        }

        command.args(config.define.iter().map(|(k, v)| match v {
            Value::Bool(true) => format!("--enable-{k}"),
            Value::Bool(false) => format!("--disable-{k}"),
            Value::String(string) => format!("--enable-{k}={string}"),
        }));

        command.args(config.include.iter().map(|(k, v)| match v {
            Value::Bool(true) => format!("--with-{k}"),
            Value::Bool(false) => format!("--without-{k}"),
            Value::String(string) => format!("--with-{k}={string}"),
        }));

        let mut child = command.spawn()?;
        let stdio = command_extra::Stdio::from_child(&mut child)
            .ok_or("Failed to extract stdio from child.")?;
        let mut lines = stdio.lines();

        tokio::spawn(async move {
            // TODO: Proper error handling!
            let _ = child.wait().await;
        });

        while let Some(line) = lines.next().await {
            match line? {
                Line::Err(line) => shell::command_err(&styles, "configure", line),
                Line::Out(line) => shell::command_out(&styles, "configure", line),
            }
        }

        let mut make = Command::new("make");

        make.current_dir(&build_dir);
        make.env_remove("CC");
        make.env_remove("CFLAGS");
        make.env_remove("CXX");
        make.env_remove("CXXFLAGS");
        make.env_remove("LIBS");

        make.arg(format!("-j{}", config.jobs))
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());

        println!("{make:?}");

        let mut child = make.spawn()?;
        let stdio = command_extra::Stdio::from_child(&mut child)
            .ok_or("Failed to extract stdio from child.")?;
        let mut lines = stdio.lines();

        tokio::spawn(async move {
            // TODO: Proper error handling!
            let _ = child.wait().await;
        });

        while let Some(line) = lines.next().await {
            match line? {
                Line::Err(line) => shell::command_err(&styles, "build", line),
                Line::Out(line) => shell::command_out(&styles, "build", line),
            }
        }

        let mut make = Command::new("make");

        make.current_dir(&build_dir);
        make.env_remove("CC");
        make.env_remove("CFLAGS");
        make.env_remove("CXX");
        make.env_remove("CXXFLAGS");
        make.env_remove("LIBS");

        make.arg("install")
            .arg(format!("-j{}", config.jobs))
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());

        println!("{make:?}");

        let mut child = make.spawn()?;
        let stdio = command_extra::Stdio::from_child(&mut child)
            .ok_or("Failed to extract stdio from child.")?;
        let mut lines = stdio.lines();

        tokio::spawn(async move {
            // TODO: Proper error handling!
            let _ = child.wait().await;
        });

        while let Some(line) = lines.next().await {
            match line? {
                Line::Err(line) => shell::command_err(&styles, "install", line),
                Line::Out(line) => shell::command_out(&styles, "install", line),
            }
        }
    }

    Ok(())
}

pub mod shell {
    use std::fmt::Display;
    use yansi::{Color, Style};

    pub struct Styles {
        decoration: &'static str,
        decoration_style: Style,
        action_style: Style,
        arguments_style: Style,
        command_style: Style,
        output_style: Style,
        output_err_style: Style,
    }

    impl Default for Styles {
        fn default() -> Self {
            Self {
                decoration: " >",
                decoration_style: Style::new(Color::White).dimmed(),
                action_style: Style::default(),
                arguments_style: Style::new(Color::Magenta),
                command_style: Style::new(Color::Green),
                output_style: Style::default(),
                output_err_style: Style::new(Color::Red),
            }
        }
    }

    pub fn header(styles: &Styles, action: impl Display, arguments: impl Display) {
        println!(
            "{decoration} {action: <13} {arguments}",
            decoration = styles.decoration_style.paint(&styles.decoration),
            action = styles.action_style.paint(&action),
            arguments = styles.arguments_style.paint(&arguments),
        );
    }

    pub fn command_out(styles: &Styles, command: impl Display, output: impl Display) {
        println!(
            "{decoration} {command} {output}",
            decoration = styles.decoration_style.paint(&styles.decoration),
            command = styles.command_style.paint(&command),
            output = styles.output_style.paint(&output),
        );
    }

    pub fn command_err(styles: &Styles, command: impl Display, output: impl Display) {
        println!(
            "{decoration} {command} {output}",
            decoration = styles.decoration_style.paint(&styles.decoration),
            command = styles.command_style.paint(&command),
            output = styles.output_err_style.paint(&output),
        );
    }
}
