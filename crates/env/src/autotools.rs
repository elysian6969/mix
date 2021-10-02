use crate::Styles;
use command_extra::Line;
use futures_util::stream::StreamExt;
use path::Path;
use std::ffi::{OsStr, OsString};
use std::process::Stdio;
use tokio::process::Command;

pub enum Kind {
    Define,
    Include,
}

pub enum Value {
    True,
    False,
    Custom(OsString),
}

pub struct Flag {
    kind: Kind,
    key: OsString,
    value: Value,
}

impl Flag {
    pub fn new(kind: Kind, key: impl AsRef<OsStr>, value: Value) -> Self {
        Self {
            kind,
            key: key.as_ref().into(),
            value,
        }
    }

    pub fn to_os_string(&self) -> OsString {
        let mut buf = OsString::new();

        match (&self.kind, &self.value) {
            (Kind::Define, Value::True) => {
                buf.push("--enable-");
                buf.push(&self.key);
            }
            (Kind::Define, Value::False) => {
                buf.push("--disable-");
                buf.push(&self.key);
            }
            (Kind::Define, Value::Custom(ref custom)) => {
                buf.push("--enable-");
                buf.push(&self.key);
                buf.push("=");
                buf.push(custom);
            }
            (Kind::Include, Value::True) => {
                buf.push("--with-");
                buf.push(&self.key);
            }
            (Kind::Include, Value::False) => {
                buf.push("--without-");
                buf.push(&self.key);
            }
            (Kind::Include, Value::Custom(ref custom)) => {
                buf.push("--with-");
                buf.push(&self.key);
                buf.push("=");
                buf.push(custom);
            }
        }

        buf
    }
}

pub async fn configure(
    styles: &Styles,
    work_dir: impl AsRef<Path>,
    destination_dir: impl AsRef<Path>,
) -> crate::Result<()> {
    let work_dir = work_dir.as_ref();
    let destination_dir = destination_dir.as_ref();
    let configure_file = work_dir.join("configure");
    let mut command = Command::new(&configure_file);

    command
        .current_dir(&work_dir)
        .env_remove("CC")
        .env_remove("CFLAGS")
        .env_remove("CXX")
        .env_remove("CXXFLAGS")
        .env_remove("LIBS")
        .arg(format!("--prefix={}", &destination_dir))
        .env("CC", "gcc")
        .env("CXX", "g++")
        .env("PREFIX", &destination_dir)
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
            Line::Err(line) => crate::shell::command_err(&styles, "configure", line),
            Line::Out(line) => crate::shell::command_out(&styles, "configure", line),
        }
    }

    Ok(())
}

/*
        if options.triple == "i686-linux-gnu" {
            command.env("CFLAGS", "-m32").env("CXXFLAGS:", "-m32");
        }

        command.args(options.define.iter().map(|(k, v)| match v {
            Value::Bool(true) => format!("--enable-{k}"),
            Value::Bool(false) => format!("--disable-{k}"),
            Value::String(string) => format!("--enable-{k}={string}"),
        }));

        command.args(options.include.iter().map(|(k, v)| match v {
            Value::Bool(true) => format!("--with-{k}"),
            Value::Bool(false) => format!("--without-{k}"),
            Value::String(string) => format!("--with-{k}={string}"),
        }));

        println!("{command:?}");

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

        make.arg(format!("-j{}", options.jobs))
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
            .arg(format!("-j{}", options.jobs))
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
}*/
