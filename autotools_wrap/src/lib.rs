use std::ffi::{OsStr, OsString};
use std::io;
use std::path::{Path, PathBuf};
use tokio::process::{Child, Command};

pub struct Aclocal {
    inner: Command,
}

impl Aclocal {
    pub(crate) fn new(source_dir: impl AsRef<Path>) -> Self {
        let mut inner = Command::new("aclocal");

        inner.current_dir(source_dir.as_ref());

        Self { inner }
    }

    pub fn include(&mut self, dir: impl AsRef<Path>) -> &mut Self {
        self.inner.arg("-I").arg(dir.as_ref());
        self
    }

    pub fn spawn(&mut self) -> io::Result<Child> {
        self.inner.spawn()
    }
}

pub fn aclocal(source_dir: impl AsRef<Path>) -> Aclocal {
    Aclocal::new(source_dir)
}

pub struct Autoconf {
    inner: Command,
}

impl Autoconf {
    pub(crate) fn new(source_dir: impl AsRef<Path>) -> Self {
        let mut inner = Command::new("autoconf");

        inner.arg("--force").current_dir(source_dir.as_ref());

        Self { inner }
    }

    pub fn spawn(&mut self) -> io::Result<Child> {
        self.inner.spawn()
    }
}

pub fn autoconf(source_dir: impl AsRef<Path>) -> Autoconf {
    Autoconf::new(source_dir)
}

pub struct Autoheader {
    inner: Command,
}

impl Autoheader {
    pub(crate) fn new(source_dir: impl AsRef<Path>) -> Self {
        let mut inner = Command::new("autoheader");

        inner.current_dir(source_dir.as_ref());

        Self { inner }
    }

    pub fn spawn(&mut self) -> io::Result<Child> {
        self.inner.spawn()
    }
}

pub fn autoheader(source_dir: impl AsRef<Path>) -> Autoheader {
    Autoheader::new(source_dir)
}

pub struct Automake {
    inner: Command,
}

impl Automake {
    pub(crate) fn new(source_dir: impl AsRef<Path>) -> Self {
        let mut inner = Command::new("automake");

        inner
            .arg("--add-missing")
            .arg("--copy")
            .arg("--foreign")
            .arg("--force-missing")
            .current_dir(source_dir.as_ref());

        Self { inner }
    }

    pub fn spawn(&mut self) -> io::Result<Child> {
        self.inner.spawn()
    }
}

pub fn automake(source_dir: impl AsRef<Path>) -> Automake {
    Automake::new(source_dir)
}

pub struct Bootstrap {
    inner: Command,
}

impl Bootstrap {
    pub(crate) fn new(source_dir: impl AsRef<Path>) -> Self {
        let mut inner = Command::new(source_dir.as_ref().join("bootstrap"));

        inner
            .arg("--skip-git")
            .arg("--skip-po")
            .current_dir(source_dir.as_ref());

        Self { inner }
    }

    pub fn spawn(&mut self) -> io::Result<Child> {
        self.inner.spawn()
    }
}

pub fn bootstrap(source_dir: impl AsRef<Path>) -> Bootstrap {
    Bootstrap::new(source_dir)
}

pub struct Configure {
    inner: Command,
}

impl Configure {
    pub(crate) fn new(source_dir: impl AsRef<Path>) -> Self {
        let mut inner = Command::new(source_dir.as_ref().join("configure"));

        inner.current_dir(source_dir.as_ref());

        Self { inner }
    }

    pub fn spawn(&mut self) -> io::Result<Child> {
        self.inner.spawn()
    }
}

pub fn configure(source_dir: impl AsRef<Path>) -> Configure {
    Configure::new(source_dir)
}
