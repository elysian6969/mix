#![allow(dead_code)]
#![allow(incomplete_features)]
#![feature(generators)]
#![feature(command_access)]
#![feature(format_args_capture)]
#![feature(iter_zip)]
#![feature(inline_const)]

use crate::compiler::{Compiler, Linker};
use crate::shell::Styles;
use command_extra::Line;
use futures_util::stream::StreamExt;
use mix_atom::Atom;
use mix_triple::Triple;
use path::{Path, PathBuf};
use std::env;
use std::process::Stdio;
use tokio::process::Command;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

mod autotools;
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

pub async fn build(_config: mix_config::Config, build_config: Config) -> Result<()> {
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

    let styles = Styles::default();

    shell::header(&styles, "prefix", &build_config.prefix);
    shell::header(&styles, "target", &build_config.target);
    shell::header(&styles, "repository_id", &repository_id);
    shell::header(&styles, "package_id", &package_id);
    shell::header(&styles, "version", &version);
    shell::header(&styles, "destination", &destination);
    shell::header(&styles, "cflags", &cflags);
    shell::header(&styles, "ldflags", &ldflags);

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
        println!("DEBUG {} -> {}", name, build_config);
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

        shell::command_out(&styles, &autogen_file, args);

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
                Line::Err(line) => shell::command_err(&styles, "autogen", line),
                Line::Out(line) => shell::command_out(&styles, "autogen", line),
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
                format!("[{}] ", styles.command_style.paint(&build_config.atom)),
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

        shell::command_out(&styles, &build_configure_file, args);*/

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
                Line::Err(line) => shell::command_err(&styles, "build_configure", line),
                Line::Out(line) => shell::command_out(&styles, "build_configure", line),
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

        shell::command_out(&styles, "make", args);

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
                Line::Err(line) => shell::command_err(&styles, "build", line),
                Line::Out(line) => shell::command_out(&styles, "build", line),
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

        shell::command_out(&styles, "make", args);

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
                Line::Err(line) => shell::command_err(&styles, "install", line),
                Line::Out(line) => shell::command_out(&styles, "install", line),
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

        shell::command_out(&styles, "make", args);

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
                Line::Err(line) => shell::command_err(&styles, "build", line),
                Line::Out(line) => shell::command_out(&styles, "build", line),
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

        shell::command_out(&styles, "cargo", args);

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
            println!("{:?}", next);
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
        pub command_style: Style,
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
