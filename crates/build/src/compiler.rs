use path::{Path, PathBuf};
use std::ffi::{OsStr, OsString};

pub struct Compiler {
    args: Vec<OsString>,
}

impl Compiler {
    pub const fn new() -> Self {
        let args = vec![];

        Self { args }
    }

    pub fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.args.push(arg.as_ref().to_os_string());
        self
    }

    pub fn args(&mut self, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> &mut Self {
        for arg in args {
            self.arg(arg);
        }

        self
    }

    fn prefixed(&mut self, pfx: impl AsRef<OsStr>, arg: impl AsRef<OsStr>) -> &mut Self {
        let pfx = pfx.as_ref();
        let arg = arg.as_ref();
        let mut buf = OsString::with_capacity(pfx.len().saturating_add(arg.len()));

        buf.push(pfx);
        buf.push(arg);

        self.arg(buf);
        self
    }

    pub fn include_dir(&mut self, dir: impl AsRef<Path>) -> &mut Self {
        self.prefixed("-I", dir.as_ref());
        self
    }

    pub fn library_dir(&mut self, dir: impl AsRef<Path>) -> &mut Self {
        self.prefixed("-L", dir.as_ref());
        self
    }

    pub fn link(&mut self, lib: impl AsRef<OsStr>) -> &mut Self {
        self.prefixed("-l", lib);
        self
    }

    pub fn file(&mut self, obj: impl AsRef<OsStr>) -> &mut Self {
        self.arg(obj);
        self
    }

    pub fn runtime_path(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.prefixed("-Wl,--rpath=", path.as_ref());
        self
    }

    pub fn dynamic_linker(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.prefixed("-Wl,--dynamic-linker=", path.as_ref());
        self
    }

    pub fn no_default_libs(&mut self) -> &mut Self {
        self.arg("-nodefaultlibs");
        self
    }

    pub fn no_start_files(&mut self) -> &mut Self {
        self.arg("-nostartfiles");
        self
    }

    pub fn pic(&mut self) -> &mut Self {
        self.arg("-fPIC");
        self
    }

    pub fn opt_level(&mut self, level: impl AsRef<OsStr>) -> &mut Self {
        self.prefixed("-O", level);
        self
    }

    pub fn target_cpu(&mut self, cpu: impl AsRef<OsStr>) -> &mut Self {
        self.prefixed("-march=", cpu);
        self
    }

    pub fn as_slice(&self) -> &[OsString] {
        &self.args[..]
    }
}

pub struct Linker {
    args: Vec<OsString>,
}

impl Linker {
    pub const fn new() -> Self {
        let args = vec![];

        Self { args }
    }

    pub fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.args.push(arg.as_ref().to_os_string());
        self
    }

    pub fn args(&mut self, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> &mut Self {
        for arg in args {
            self.arg(arg);
        }

        self
    }

    fn prefixed(&mut self, pfx: impl AsRef<OsStr>, arg: impl AsRef<OsStr>) -> &mut Self {
        let pfx = pfx.as_ref();
        let arg = arg.as_ref();
        let mut buf = OsString::with_capacity(pfx.len().saturating_add(arg.len()));

        buf.push(pfx);
        buf.push(arg);

        self.arg(buf);
        self
    }

    pub fn library_dir(&mut self, dir: impl AsRef<Path>) -> &mut Self {
        self.prefixed("-L", dir.as_ref());
        self
    }

    pub fn file(&mut self, obj: impl AsRef<OsStr>) -> &mut Self {
        self.arg(obj);
        self
    }

    pub fn runtime_path(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.prefixed("-Wl,--rpath=", path.as_ref());
        self
    }

    pub fn dynamic_linker(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.prefixed("-Wl,--dynamic-linker=", path.as_ref());
        self
    }

    pub fn as_slice(&self) -> &[OsString] {
        &self.args[..]
    }
}
