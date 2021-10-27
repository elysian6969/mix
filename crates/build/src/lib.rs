#![allow(dead_code)]
#![allow(incomplete_features)]
#![feature(generators)]
#![feature(format_args_capture)]
#![feature(iter_zip)]
#![feature(inline_const)]
#![feature(format_args_nl)]
#![feature(option_result_unwrap_unchecked)]
#![feature(iter_intersperse)]

use self::process::Command;
use crate::compiler::{Compiler, Linker};
use command_extra::{Line, Stdio};
use mix_atom::{Atom, Requirement};
use mix_id::RepositoryId;
use mix_packages::{Package, Packages};
use mix_shell::{header, writeln, AsyncWrite};
use mix_source::Kind;
use mix_triple::Triple;
use path::{Path, PathBuf};
use std::borrow::Borrow;
use std::borrow::Cow;
use std::env;
use std::sync::Arc;

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
    pub requirement: Requirement,
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

use std::collections::HashSet;

fn resolve(
    config: &mix_config::Config,
    packages: &Arc<Packages>,
    dependencies: &mut Vec<Package>,
    exists: &mut HashSet<Package>,
    requirement: &Requirement,
) {
    let matches = packages.matches(requirement).collect::<Vec<_>>();

    if matches.len() != 1 {
        return;
    }

    // SAFETY: above clause requires at least 1 package to be present.
    let package = unsafe { matches.get_unchecked(0) };

    if exists.contains(Borrow::<Package>::borrow(package)) {
        return;
    }

    dependencies.push(Package::clone(package));
    exists.insert(Package::clone(package));

    for requirement in package.dependencies() {
        resolve(config, packages, dependencies, exists, requirement);
    }
}

pub async fn build(
    config: mix_config::Config,
    build_config: Config,
    packages: Arc<Packages>,
) -> Result<()> {
    let mut dependencies = vec![];
    let mut exists = HashSet::new();

    resolve(
        &config,
        &packages,
        &mut dependencies,
        &mut exists,
        &build_config.requirement,
    );

    let dependencies = dependencies.into_iter().rev().collect::<Vec<_>>();

    for dependency in dependencies.iter() {
        println!("{}/{}", dependency.repository_id(), dependency.package_id());
    }

    let mut sources = vec![];
    let mut exists = HashSet::new();
    let mut iter = dependencies
        .iter()
        .flat_map(|dependency| dependency.sources().iter());

    for source in iter {
        if !exists.contains(source) {
            sources.push(source);
            exists.insert(source);
        }
    }

    for source in sources {
        source.update(&config).await?;

        let versions = source.versions(&config).await?;

        if let Some(entry) = versions.latest() {
            config.download_file(&entry.path, &entry.url).await?;
        }
    }

    return Ok(());

    /*
    // SAFETY: above clause requires at least 1 package to be present.
    let package = unsafe { matches.get_unchecked(0) };
    let repository_id = package.repository_id();
    let package_id = package.package_id();
    let version = &build_config.atom.version;
    let target_str = build_config.target.as_str();
    let version_str = version.to_string();
    let version_str = version_str.as_str();

    let build_dir = config
        .build_prefix()
        .join(target_str)
        .join(&repository_id)
        .join(&package_id)
        .join(&version_str);

    let cache_dir = config.cache_prefix().join(&repository_id);

    let destination = build_config
        .prefix
        .join(target_str)
        .join(&repository_id)
        .join(&package_id)
        .join(&version_str);

    let libc_root = build_config
        .prefix
        .join(target_str)
        .join(&repository_id)
        .join("glibc")
        .join("2.34.0");

    let libc_lib = libc_root.join("lib");
    let dynamic_linker = format!("ld-linux-{}.so.2", build_config.target.arch_str());

    let _compiler_root = build_config
        .prefix
        .join(build_config.target.as_str())
        .join(&repository_id)
        .join("gcc")
        .join("11.2.0");

    let current_dir: PathBuf = env::current_dir()?.into();
    let build_dir = if build_config.build_dir {
        build_dir.join("build")
    } else {
        build_dir
    };

    let _ = build_dir.create_dir_all_async().await;

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
        .library_dir("/milk/x86_64-linux-gnu/core/gcc/11.2.0/lib/gcc/x86_64-pc-linux-gnu/11.2.0")
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
        .intersperse(Cow::Borrowed(" "))
        .collect::<String>();

    let ldflags = linker
        .as_slice()
        .iter()
        .map(|s| s.to_string_lossy())
        .intersperse(Cow::Borrowed(" "))
        .collect::<String>();

    writeln!(config.shell(), "{:?}", package);

    writeln!(config.shell(), "installed: {:?}", package.installed());

    header!(
        config.shell(),
        "building {}/{}:{}",
        package.repository_id(),
        package.package_id(),
        &build_config.atom.version,
    )?;

    header!(config.shell(), "destination {}", &destination)?;
    header!(config.shell(), "build {}", &build_dir)?;
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
            .c_compiler(GCC)
            .c_flags(&cflags)
            .cxx_compiler(GXX)
            .cxx_flags(&cflags)
            .current_dir(&build_dir)
            .home_dir(&current_dir)
            .lang(EN_US)
            .linker(LD)
            .linker_flags(&ldflags)
            .paths(["/milk/global/bin"])
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

            / *match line? {
                Line::Err(line) => writeln!(config.shell(), "{} {}", "configure", line)?,
                Line::Out(line) => writeln!(config.shell(), "{} {}", "configure", line)?,
            }* /
        }
    }*/

    Ok(())
}
