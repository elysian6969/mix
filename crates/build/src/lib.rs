#![allow(dead_code)]
#![allow(incomplete_features)]
#![feature(generators)]
#![feature(format_args_capture)]
#![feature(iter_zip)]
#![feature(inline_const)]
#![feature(format_args_nl)]
#![feature(option_result_unwrap_unchecked)]

use self::process::Command;
use crate::compiler::{Compiler, Linker};
use command_extra::{Line, Stdio};
use mix_atom::Atom;
use mix_shell::{header, writeln, AsyncWrite};
use mix_triple::Triple;
use path::{Path, PathBuf};
use std::env;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

//mod autotools;
mod compiler;
mod configs;
mod process;

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

pub(crate) const CLANG: &str = "clang";
pub(crate) const CLANGXX: &str = "clang++";
pub(crate) const GCC: &str = "gcc";
pub(crate) const GXX: &str = "g++";
pub(crate) const EN_US: &str = "en_US.UTF-8";
pub(crate) const LD: &str = "ld";
pub(crate) const LLD: &str = "ld.lld";

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

    header!(config.shell(), "building {}", &build_config.atom)?;
    header!(config.shell(), "destination {}", &destination)?;
    header!(config.shell(), "cflags {}", &cflags)?;
    header!(config.shell(), "ldflags {}", &ldflags)?;

    let build = configs::System::new(package_id, &current_dir).await;

    for (_name, build_config) in build.config.iter() {
        header!(config.shell(), "found {}", build_config)?;
    }

    if let Some((Some(bootstrap), _)) = build.get_autotools() {
        let mut command = Command::new(bootstrap);

        command
            .c_compiler(CLANG)
            .c_flags(&cflags)
            .cxx_compiler(CLANGXX)
            .cxx_flags(&cflags)
            .current_dir(&build_dir)
            .home_dir(&current_dir)
            .lang(EN_US)
            .linker(LLD)
            .linker_flags(&ldflags)
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());

        let mut child = command.spawn().await?;
        let mut stdio = child.stdio()?.expect("stdio");
        let mut lines = stdio.lines();

        tokio::spawn(async move {
            // TODO: Proper error handling!
            let _ = child.wait().await;
        });

        while let Some(line) = lines.next_line().await {
            match line? {
                Line::Err(line) => writeln!(config.shell(), "{} {}", "bootstrap", line)?,
                Line::Out(line) => writeln!(config.shell(), "{} {}", "bootstrap", line)?,
            }
        }
    }

    if let Some((_, Some(configure))) = build.get_autotools() {
        let mut command = Command::new(configure);

        command
            .env_clear()
            .env("PATH", "/milk/global/bin")
            //.arg(format!("--prefix={}", destination))
            .c_compiler(GCC)
            .c_flags(&cflags)
            .cxx_compiler(GXX)
            .cxx_flags(&cflags)
            .current_dir(&build_dir)
            .home_dir(&current_dir)
            .lang(EN_US)
            .linker(LD)
            .linker_flags(&ldflags)
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());

        let mut child = command.spawn().await?;
        let mut stdio = child.stdio()?.expect("stdio");
        let mut lines = stdio.lines();

        tokio::spawn(async move {
            // TODO: Proper error handling!
            let _ = child.wait().await;
        });

        while let Some(line) = lines.next_line().await {
            writeln!(config.shell(), "{} {:?}", "configure", line)?;

            /*match line? {
                Line::Err(line) => writeln!(config.shell(), "{} {}", "configure", line)?,
                Line::Out(line) => writeln!(config.shell(), "{} {}", "configure", line)?,
            }*/
        }
    }

    Ok(())
}
