#![allow(dead_code)]
#![allow(incomplete_features)]
#![feature(generators)]
#![feature(format_args_capture)]
#![feature(iter_zip)]
#![feature(inline_const)]
#![feature(format_args_nl)]

use crate::compiler::{Compiler, Linker};
use command_extra::Line;
use futures_util::stream::StreamExt;
use mix_atom::Atom;
use mix_shell::{header, writeln, AsyncWrite};
use mix_triple::Triple;
use path::{Path, PathBuf};
use std::env;
use std::process::Stdio;
use tokio::process::Command;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

//mod autotools;
mod compiler;
mod configs;

#[derive(Debug)]
pub enum Value {
    Bool(bool),
    String(String),
}

#[derive(Debug)]
pub struct Config {
    pub prefix: PathBuf,
    pub target: Triple,
    pub atom: Atom,
    pub jobs: usize,
    pub define: Vec<(String, Value)>,
    pub include: Vec<(String, Value)>,
    pub build_dir: bool,
}

pub async fn build(config: mix_config::Config, build_config: Config) -> Result<()> {
    let core = "core".try_into()?;
    let repository_id = (&build_config.atom.repository_id).as_ref().unwrap_or(&core);
    let package_id = &build_config.atom.package_id;
    let version = &build_config.atom.version;

    let destination = build_config
        .prefix
        .join(build_config.target.as_str())
        .join(repository_id.as_str())
        .join(package_id.as_str())
        .join(version.to_string());

    let libc_root = build_config
        .prefix
        .join(build_config.target.as_str())
        .join(repository_id.as_str())
        .join("glibc")
        .join("2.34.0");

    let libc_lib = libc_root.join("lib");

    let dynamic_linker = format!("ld-linux-{}.so.2", build_config.target.arch_str());

    let _compiler_root = build_config
        .prefix
        .join(build_config.target.as_str())
        .join(repository_id.as_str())
        .join("gcc")
        .join("11.2.0");

    let current_dir: PathBuf = env::current_dir()?.into();
    let build_dir = if build_config.build_dir {
        let build_dir = PathBuf::from("/mix/build");

        // TODO: Proper error handling,
        let _ = build_dir.create_dir_async().await;

        build_dir
    } else {
        current_dir.clone()
    };

    // NOTE: autotools appears to be retarded
    // compiler.file("/mix/x86_64-linux-gnu/core/glibc/2.34.0/lib/crti.o")

    let mut compiler = Compiler::new();
    let mut linker = Linker::new();

    compiler.opt_level("fast");

    if matches!(
        build_config.target,
        const { Triple::i686() } | const { Triple::x86_64() }
    ) {
        compiler.target_cpu("native");
    }

    compiler
        .no_default_libs()
        .no_start_files()
        .pic()
        .linker("lld")
        .library_dir("/mix/x86_64-linux-gnu/core/gcc/11.2.0/lib/gcc/x86_64-pc-linux-gnu/11.2.0")
        .library_dir(&libc_lib)
        .file(libc_lib.join("crt1.o"))
        .file(libc_lib.join("crtn.o"))
        .link("gcc")
        .link("c")
        .runtime_path(&libc_lib)
        .dynamic_linker(libc_lib.join(&dynamic_linker));

    linker
        .runtime_path(&libc_lib)
        .dynamic_linker(libc_lib.join(dynamic_linker));

    let cflags = compiler
        .as_slice()
        .iter()
        .map(|s| s.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");

    let ldflags = linker
        .as_slice()
        .iter()
        .map(|s| s.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");

    header!(config.shell(), "prefix {}", &build_config.prefix)?;
    header!(config.shell(), "target {}", &build_config.target)?;
    header!(config.shell(), "repository_id {}", &repository_id)?;
    header!(config.shell(), "package_id {}", &package_id)?;
    header!(config.shell(), "version {}", &version)?;
    header!(config.shell(), "destination {}", &destination)?;
    header!(config.shell(), "cflags {}", &cflags)?;
    header!(config.shell(), "ldflags {}", &ldflags)?;

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

    let build_configs = configs::detect(package_id, &current_dir).await;

    for (name, build_config) in build_configs.iter() {
        writeln!(config.shell(), "DEBUG {} -> {}", name, build_config)?;
    }

    if let Some(autogen_file) = build_configs
        .get("autogen")
        .or_else(|| build_configs.get("autogen.sh"))
    {
        let mut command = std::process::Command::new(&autogen_file);

        command
            .current_dir(&build_dir)
            .arg(format!("--prefix={}", &destination))
            .env("CC", "clang")
            .env("CFLAGS", &cflags)
            .env("CXX", "clang++")
            .env("CXXFLAGS", &cflags)
            .env("HOME", &current_dir)
            .env("LANG", "en_US.UTF-8")
            .env("LD", "ld.lld")
            .env("LDFLAGS", &ldflags)
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());

        let args: Vec<_> = command.get_args().flat_map(|arg| arg.to_str()).collect();
        let args: String = args.join(" ");

        writeln!(config.shell(), "{}{}", &autogen_file, args)?;

        let mut command = Command::from(command);
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
                Line::Err(line) => writeln!(config.shell(), "{}{}", "autogen", line)?,
                Line::Out(line) => writeln!(config.shell(), "{}{}", "autogen", line)?,
            }
        }
    }

    if let Some(_build_configure_file) = build_configs
        .get("build_configure")
        .or_else(|| build_configs.get("build_configure.sh"))
    {
        //let mut command = std::process::Command::new(&build_configure_file);
        let mut command = std::process::Command::new("sh");

        command
            .current_dir(&build_dir)
            //.arg(format!("--prefix={}", &destination))
            .env_clear()
            .env("CC", "clang")
            .env("CFLAGS", &cflags)
            .env("CXX", "clang++")
            .env("CXXFLAGS", &cflags)
            .env("HOME", &current_dir)
            .env(
                "PS1",
                format!(
                    "[{}] ",
                    config.shell().theme().command_paint(&build_config.atom)
                ),
            )
            .env("LANG", "en_US.UTF-8")
            .env("LD", "ld.lld")
            .env("LDFLAGS", &ldflags);
        /*.stderr(Stdio::piped())
        .stdin(Stdio::null())
        .stdout(Stdio::piped());*/

        if build_config.target == Triple::i686() {
            command.env("CFLAGS", "-m32");
            command.env("CXXFLAGS", "-m32");
        }

        /*command.args(build_config.define.iter().map(|(k, v)| match v {
            Value::Bool(true) => format!("--enable-{k}"),
            Value::Bool(false) => format!("--disable-{k}"),
            Value::String(string) => format!("--enable-{k}={string}"),
        }));

        command.args(build_config.include.iter().map(|(k, v)| match v {
            Value::Bool(true) => format!("--with-{k}"),
            Value::Bool(false) => format!("--without-{k}"),
            Value::String(string) => format!("--with-{k}={string}"),
        }));

        let args: Vec<_> = command.get_args().flat_map(|arg| arg.to_str()).collect();
        let args: String = args.join(" ");

        shell::command_out(config.shell(), &build_configure_file, args);*/

        let mut command = Command::from(command);
        let mut child = command.spawn()?;
        /*let stdio = command_extra::Stdio::from_child(&mut child)
            .ok_or("Failed to extract stdio from child.")?;
        let mut lines = stdio.lines();*/

        //tokio::spawn(async move {
        // TODO: Proper error handling!
        let _ = child.wait().await;
        //});

        /*while let Some(line) = lines.next().await {
            match line? {
                Line::Err(line) => shell::command_err(config.shell(), "build_configure", line),
                Line::Out(line) => shell::command_out(config.shell(), "build_configure", line),
            }
        }

        let mut make = std::process::Command::new("make");

        make.current_dir(&build_dir);

        make.arg(format!("-j{}", build_config.jobs))
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());

        let args: Vec<_> = make.get_args().flat_map(|arg| arg.to_str()).collect();
        let args: String = args.join(" ");

        shell::command_out(config.shell(), "make", args);

        let mut make = Command::from(make);
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
                Line::Err(line) => shell::command_err(config.shell(), "build", line),
                Line::Out(line) => shell::command_out(config.shell(), "build", line),
            }
        }

        let mut make = std::process::Command::new("make");

        make.current_dir(&build_dir);

        make.arg("install")
            .arg(format!("-j{}", build_config.jobs))
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());

        let args: Vec<_> = make.get_args().flat_map(|arg| arg.to_str()).collect();
        let args: String = args.join(" ");

        shell::command_out(config.shell(), "make", args);

        let mut make = Command::from(make);
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
                Line::Err(line) => shell::command_err(config.shell(), "install", line),
                Line::Out(line) => shell::command_out(config.shell(), "install", line),
            }
        }*/

        return Ok(());
    }

    if let Some(_makefile) = build_configs
        .get("makefile")
        .or_else(|| build_configs.get("Makefile"))
        .or_else(|| build_configs.get("gnumakefile"))
        .or_else(|| build_configs.get("GNUmakefile"))
        .or_else(|| build_configs.get("GNUMakefile"))
    {
        let mut command = std::process::Command::new("make");

        command
            .arg(format!("--jobs={}", build_config.jobs))
            .env("PREFIX", &destination)
            .env("CC", "clang")
            .env("CFLAGS", &cflags)
            .env("CXX", "clang++")
            .env("CXXFLAGS", &cflags)
            .env("HOME", &current_dir)
            .env("LANG", "en_US.UTF-8")
            .env("LD", "ld.lld")
            .env("LDFLAGS", &ldflags)
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());

        let args: Vec<_> = command.get_args().flat_map(|arg| arg.to_str()).collect();
        let args: String = args.join(" ");

        writeln!(config.shell(), "{}{}", "make", args)?;

        let mut command = Command::from(command);
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
                Line::Err(line) => writeln!(config.shell(), "{}{}", "build", line)?,
                Line::Out(line) => writeln!(config.shell(), "{}{}", "build", line)?,
            }
        }
    }

    if build_configs.get("Cargo.toml").is_some() {
        let mut command = std::process::Command::new("cargo");

        command
            .arg("build")
            .arg(format!("--jobs={}", build_config.jobs))
            .arg("--release")
            .env("CC", "clang")
            .env("CFLAGS", &cflags)
            .env("CXX", "clang++")
            .env("CXXFLAGS", &cflags)
            .env("HOME", &current_dir)
            .env("LANG", "en_US.UTF-8")
            .env("LD", "ld.lld")
            .env("LDFLAGS", &ldflags)
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());

        let args: Vec<_> = command.get_args().flat_map(|arg| arg.to_str()).collect();
        let args: String = args.join(" ");

        writeln!(config.shell(), "{}{}", "cargo", args)?;

        let mut command = Command::from(command);
        let mut child = command.spawn()?;
        let stdio = command_extra::Stdio::from_child(&mut child)
            .ok_or("Failed to extract stdio from child.")?;
        let mut lines = stdio.lines();

        tokio::spawn(async move {
            // TODO: Proper error handling!
            let _ = child.wait().await;
        });

        while let Some(next) = lines.next().await {
            writeln!(config.shell(), "{:?}", next)?;
        }
    }

    Ok(())
}
