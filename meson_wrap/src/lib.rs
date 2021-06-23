use std::ffi::{OsStr, OsString};
use std::io;
use std::path::{Path, PathBuf};
use tokio::process::{Child, Command};

pub(crate) struct Inner {
    inner: Command,
}

impl Inner {
    pub fn new() -> Self {
        let inner = Command::new("meson");

        Self { inner }
    }

    pub fn setup(&mut self) -> &mut Self {
        self.inner.arg("setup");
        self
    }

    pub fn configure(&mut self) -> &mut Self {
        self.inner.arg("configure");
        self
    }

    pub fn dist(&mut self) -> &mut Self {
        self.inner.arg("dist");
        self
    }

    pub fn install(&mut self) -> &mut Self {
        self.inner.arg("install");
        self
    }

    pub fn introspect(&mut self) -> &mut Self {
        self.inner.arg("introspect");
        self
    }

    pub fn init(&mut self) -> &mut Self {
        self.inner.arg("init");
        self
    }

    pub fn test(&mut self) -> &mut Self {
        self.inner.arg("test");
        self
    }

    pub fn wrap(&mut self) -> &mut Self {
        self.inner.arg("wrap");
        self
    }

    pub fn subprojects(&mut self) -> &mut Self {
        self.inner.arg("subprojects");
        self
    }

    pub fn rewrite(&mut self) -> &mut Self {
        self.inner.arg("rewrite");
        self
    }

    pub fn compile(&mut self) -> &mut Self {
        self.inner.arg("compile");
        self
    }

    pub fn current_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.current_dir(path);
        self
    }

    pub fn build_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.arg(path.as_ref());
        self.current_dir(path);
        self
    }

    pub fn source_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.arg(path.as_ref());
        self
    }

    fn keyval(
        &mut self,
        flag: impl AsRef<OsStr>,
        key: impl AsRef<OsStr>,
        value: impl AsRef<OsStr>,
    ) -> &mut Self {
        let mut arg = flag.as_ref().to_os_string();

        arg.push(key.as_ref());
        arg.push("=");
        arg.push(value.as_ref());

        self.inner.arg(arg);
        self
    }

    pub fn define(&mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> &mut Self {
        self.keyval("-D", key, value);
        self
    }

    pub fn tests(&mut self, value: bool) -> &mut Self {
        self.define("tests", value.to_string());
        self
    }

    pub fn build_kind(&mut self, kind: impl AsRef<OsStr>) -> &mut Self {
        self.keyval("--", "buildtype", kind);
        self
    }

    pub fn wrap_kind(&mut self, kind: impl AsRef<OsStr>) -> &mut Self {
        self.keyval("--", "wrap-mode", kind);
        self
    }

    pub fn prefix_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.keyval("--", "prefix", path.as_ref());
        self
    }

    pub fn bin_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.keyval("--", "bindir", path.as_ref());
        self
    }

    pub fn data_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.keyval("--", "datadir", path.as_ref());
        self
    }

    pub fn include_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.keyval("--", "includedir", path.as_ref());
        self
    }

    pub fn info_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.keyval("--", "infodir", path.as_ref());
        self
    }

    pub fn lib_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.keyval("--", "libdir", path.as_ref());
        self
    }

    pub fn libexec_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.keyval("--", "libexecdir", path.as_ref());
        self
    }

    pub fn locale_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.keyval("--", "localedir", path.as_ref());
        self
    }

    pub fn localstate_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.keyval("--", "localstatedir", path.as_ref());
        self
    }

    pub fn man_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.keyval("--", "mandir", path.as_ref());
        self
    }

    pub fn sbin_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.keyval("--", "sbindir", path.as_ref());
        self
    }

    pub fn sharedstate_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.keyval("--", "sharedstatedir", path.as_ref());
        self
    }

    pub fn sysconf_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.keyval("--", "sysconfdir", path.as_ref());
        self
    }

    pub fn spawn(&mut self) -> io::Result<Child> {
        self.inner.spawn()
    }
}

pub struct Configure {
    inner: Inner,
    build_dir: PathBuf,
    source_dir: PathBuf,
}

impl Configure {
    pub(crate) fn new(source_dir: impl AsRef<Path>, build_dir: impl AsRef<Path>) -> Self {
        let mut inner = Inner::new();
        let build_dir = build_dir.as_ref().to_path_buf();
        let source_dir = source_dir.as_ref().to_path_buf();

        inner.setup();

        Self {
            inner,
            build_dir,
            source_dir,
        }
    }

    pub fn define(&mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> &mut Self {
        self.inner.define(key, value);
        self
    }

    pub fn tests(&mut self, value: bool) -> &mut Self {
        self.inner.tests(value);
        self
    }

    pub fn build_kind(&mut self, kind: impl AsRef<OsStr>) -> &mut Self {
        self.inner.build_kind(kind);
        self
    }

    pub fn wrap_kind(&mut self, kind: impl AsRef<OsStr>) -> &mut Self {
        self.inner.wrap_kind(kind);
        self
    }

    pub fn prefix_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.prefix_dir(path);
        self
    }

    pub fn bin_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.bin_dir(path);
        self
    }

    pub fn data_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.data_dir(path);
        self
    }

    pub fn include_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.include_dir(path);
        self
    }

    pub fn info_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.info_dir(path);
        self
    }

    pub fn lib_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.lib_dir(path);
        self
    }

    pub fn libexec_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.libexec_dir(path);
        self
    }

    pub fn locale_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.locale_dir(path);
        self
    }

    pub fn localstate_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.localstate_dir(path);
        self
    }

    pub fn man_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.man_dir(path);
        self
    }

    pub fn sbin_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.sbin_dir(path);
        self
    }

    pub fn sharedstate_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.sharedstate_dir(path);
        self
    }

    pub fn sysconf_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.sysconf_dir(path);
        self
    }

    pub fn spawn(&mut self) -> io::Result<Child> {
        self.inner.build_dir(&self.build_dir);
        self.inner.source_dir(&self.source_dir);
        self.inner.spawn()
    }
}

pub fn configure(source_dir: impl AsRef<Path>, build_dir: impl AsRef<Path>) -> Configure {
    Configure::new(source_dir, build_dir)
}

pub struct Build {
    inner: Inner,
}

impl Build {
    pub(crate) fn new(build_dir: impl AsRef<Path>) -> Self {
        let mut inner = Inner::new();

        inner.compile();
        inner.current_dir(build_dir);

        Self { inner }
    }

    pub fn spawn(&mut self) -> io::Result<Child> {
        self.inner.spawn()
    }
}

pub fn build(build_dir: impl AsRef<Path>) -> Build {
    Build::new(build_dir)
}

pub struct Install {
    inner: Inner,
}

impl Install {
    pub(crate) fn new(build_dir: impl AsRef<Path>) -> Self {
        let mut inner = Inner::new();

        inner.install();
        inner.current_dir(build_dir);

        Self { inner }
    }

    pub fn spawn(&mut self) -> io::Result<Child> {
        self.inner.spawn()
    }
}

pub fn install(build_dir: impl AsRef<Path>) -> Install {
    Install::new(build_dir)
}
