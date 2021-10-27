#![allow(dead_code)]
#![allow(incomplete_features)]
#![feature(generators)]
#![feature(format_args_capture)]
#![feature(iter_zip)]
#![feature(inline_const)]

use crate::compiler::{Compiler, Linker};
use crate::shell::Styles;
use mix_atom::Atom;
use mix_triple::Triple;
use path::PathBuf;
use std::env;
use tokio::process::Command;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

mod compiler;

#[derive(Debug)]
pub struct Config {
    pub prefix: PathBuf,
    pub target: Triple,
    pub atom: Atom,
}

pub async fn env(_gconfig: mix_config::Config, config: Config) -> Result<()> {
    let core = "core".try_into()?;
    let repository_id = (&config.atom.repository_id).as_ref().unwrap_or(&core);
    let package_id = &config.atom.package_id;
    let version = &config.atom.version;

    let destination = config
        .prefix
        .join(config.target.as_str())
        .join(repository_id.as_str())
        .join(package_id.as_str())
        .join(version.to_string());

    let libc_root = config
        .prefix
        .join(config.target.as_str())
        .join(repository_id.as_str())
        .join("glibc")
        .join("2.34.0");

    let libc_lib = libc_root.join("lib");

    let dynamic_linker = format!("ld-linux-{}.so.2", config.target.arch_str());

    let _compiler_root = config
        .prefix
        .join(config.target.as_str())
        .join(repository_id.as_str())
        .join("gcc")
        .join("11.2.0");

    let current_dir: PathBuf = env::current_dir()?.into();

    // NOTE: autotools appears to be retarded
    // compiler.file("/milk/x86_64-linux-gnu/core/glibc/2.34.0/lib/crti.o")

    let mut compiler = Compiler::new();
    let mut linker = Linker::new();

    compiler.opt_level("fast");

    if matches!(
        config.target,
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
        .collect::<Vec<_>>()
        .join(" ");

    let ldflags = linker
        .as_slice()
        .iter()
        .map(|s| s.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");

    let styles = Styles::default();

    shell::header(&styles, "prefix", &config.prefix);
    shell::header(&styles, "target", &config.target);
    shell::header(&styles, "repository_id", &repository_id);
    shell::header(&styles, "package_id", &package_id);
    shell::header(&styles, "version", &version);
    shell::header(&styles, "destination", &destination);
    shell::header(&styles, "cflags", &cflags);
    shell::header(&styles, "ldflags", &ldflags);

    let mut command = std::process::Command::new("sh");
    let ps1 = format!("[{}] ", styles.command_style.paint(&config.atom));

    command
        .current_dir(&current_dir)
        .env_clear()
        .env("CC", "clang")
        .env("CFLAGS", &cflags)
        .env("CXX", "clang++")
        .env("CXXFLAGS", &cflags)
        .env("HOME", &current_dir)
        .env("PS1", ps1)
        .env("LANG", "en_US.UTF-8")
        .env("LD", "ld.lld")
        .env("LDFLAGS", &ldflags);

    if config.target == Triple::i686() {
        command.env("CFLAGS", "-m32");
        command.env("CXXFLAGS", "-m32");
    }

    let mut command = Command::from(command);
    let mut child = command.spawn()?;
    let _ = child.wait().await;

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
