use std::ffi::{OsStr, OsString};
use std::io;
use std::path::Path;
use tokio::process::{Child, Command};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Generator {
    UnixMakefiles,
    GreenHillsMulti,
    Ninja,
    NinjaMultiConfig,
    WatcomWMake,
    CodeBlocksNinja,
    CodeBlocksUnixMakefiles,
    CodeLiteNinja,
    CodeLiteUnixMakefiles,
    SublimeText2Ninja,
    SublimeText2UnixMakefiles,
    KateNinja,
    KateUnixMakefiles,
    EclipseCDT4Ninja,
    EclipseCDT4UnixMakefiles,
}

impl Generator {
    pub const fn as_str(&self) -> &'static str {
        use Generator::*;

        match *self {
            UnixMakefiles => "Unix Makefiles",
            GreenHillsMulti => "Green Hills MULTI",
            Ninja => "Ninja",
            NinjaMultiConfig => "Ninja Multi-Config",
            WatcomWMake => "Watcom WMake",
            CodeBlocksNinja => "CodeBlocks - Ninja",
            CodeBlocksUnixMakefiles => "CodeBlocks - Unix Makefiles",
            CodeLiteNinja => "CodeLite - Ninja",
            CodeLiteUnixMakefiles => "CodeBlocks - Unix Makefiles",
            SublimeText2Ninja => "Sublime Text 2 - Ninja",
            SublimeText2UnixMakefiles => "Sublime Text 2 - Unix Makefiles",
            KateNinja => "Kate - Ninja",
            KateUnixMakefiles => "Kate - Unix Makefiles",
            EclipseCDT4Ninja => "Eclipse CDT4 - Ninja",
            EclipseCDT4UnixMakefiles => "Eclipse CDT4 - Unix Makefiles",
        }
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self::UnixMakefiles
    }
}

struct Inner {
    inner: Command,
}

impl Inner {
    pub fn new() -> Self {
        let inner = Command::new("cmake");

        Self { inner }
    }

    pub fn generator(&mut self, generator: Generator) -> &mut Self {
        self.inner.arg("-G").arg(generator.as_str());
        self
    }

    pub fn build(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.arg("--build").arg(path.as_ref());
        self
    }

    pub fn install(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.arg("--install").arg(path.as_ref());
        self
    }

    pub fn jobs(&mut self, jobs: usize) -> &mut Self {
        self.inner.arg("--parallel").arg(jobs.to_string());
        self
    }

    pub fn build_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.arg("-B").arg(path.as_ref());
        self
    }

    pub fn source_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.arg("-S").arg(path.as_ref());
        self
    }

    pub fn define(&mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> &mut Self {
        let mut def = OsString::from("-D");

        def.push(key.as_ref());
        def.push("=");
        def.push(value.as_ref());

        self.inner.arg(def);
        self
    }

    pub fn cmake_install_prefix_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.define("CMAKE_INSTALL_PREFIX", path.as_ref());
        self
    }

    /// user executables (bin)
    pub fn cmake_install_bin_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.define("CMAKE_INSTALL_BINDIR", path.as_ref());
        self
    }

    /// system admin executables (sbin)
    pub fn cmake_install_sbin_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.define("CMAKE_INSTALL_SBINDIR", path.as_ref());
        self
    }

    /// program executables (libexec)
    pub fn cmake_install_libexec_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.define("CMAKE_INSTALL_LIBEXECDIR", path.as_ref());
        self
    }

    /// read-only single-machine data (etc)
    pub fn cmake_install_sysconf_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.define("CMAKE_INSTALL_SYSCONFDIR", path.as_ref());
        self
    }

    /// modifiable architecture-independent data (com)
    pub fn cmake_install_sharedstate_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.define("CMAKE_INSTALL_SHAREDSTATEDIR", path.as_ref());
        self
    }

    /// modifiable single-machine data (var)
    pub fn cmake_install_localstate_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.define("CMAKE_INSTALL_LOCALSTATEDIR", path.as_ref());
        self
    }

    /// run-time variable data ({localstate}/run)
    pub fn cmake_install_runstate_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.define("CMAKE_INSTALL_RUNSTATEDIR", path.as_ref());
        self
    }

    /// object code libraries (lib or lib64 or lib/<multiarch-tuple>)
    pub fn cmake_install_lib_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.define("CMAKE_INSTALL_LIBDIR", path.as_ref());
        self
    }

    /// C header files (include)
    pub fn cmake_install_include_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.define("CMAKE_INSTALL_INCLUDEDIR", path.as_ref());
        self
    }

    /// C header files for non-gcc (/usr/include)
    pub fn cmake_install_oldinclude_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.define("CMAKE_INSTALL_OLDINCLUDEDIR", path.as_ref());
        self
    }

    /// read-only architecture-independent data root (share)
    pub fn cmake_install_dataroot_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.define("CMAKE_INSTALL_DATAROOTDIR", path.as_ref());
        self
    }

    /// read-only architecture-independent data ({dataroot})
    pub fn cmake_install_data_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.define("CMAKE_INSTALL_DATADIR", path.as_ref());
        self
    }

    /// info documentation ({dataroot}/info)
    pub fn cmake_install_info_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.define("CMAKE_INSTALL_INFODIR", path.as_ref());
        self
    }

    /// locale-dependent data ({dataroot}/locale)
    pub fn cmake_install_locale_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.define("CMAKE_INSTALL_LOCALEDIR", path.as_ref());
        self
    }

    /// man documentation ({dataroot}/man)
    pub fn cmake_install_man_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.define("CMAKE_INSTALL_MANDIR", path.as_ref());
        self
    }

    /// documentation root ({dataroot}/doc/{project})
    pub fn cmake_install_doc_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.define("CMAKE_INSTALL_DOCDIR", path.as_ref());
        self
    }

    pub fn spawn(&mut self) -> io::Result<Child> {
        self.inner.spawn()
    }
}

pub struct Configure {
    inner: Inner,
}

impl Configure {
    pub fn new() -> Self {
        let inner = Inner::new();

        Self { inner }
    }

    pub fn generator(&mut self, generator: Generator) -> &mut Self {
        self.inner.generator(generator);
        self
    }

    pub fn build_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.build_dir(path);
        self
    }

    pub fn source_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.source_dir(path);
        self
    }

    pub fn define(&mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> &mut Self {
        self.inner.define(key, value);
        self
    }

    pub fn prefix_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.cmake_install_prefix_dir(path);
        self
    }

    pub fn bin_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.cmake_install_bin_dir(path);
        self
    }

    pub fn sbin_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.cmake_install_sbin_dir(path);
        self
    }

    pub fn libexec_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.cmake_install_libexec_dir(path);
        self
    }

    pub fn sysconf_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.cmake_install_sysconf_dir(path);
        self
    }

    pub fn sharedstate_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.cmake_install_sharedstate_dir(path);
        self
    }

    pub fn localstate_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.cmake_install_localstate_dir(path);
        self
    }

    pub fn runstate_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.cmake_install_runstate_dir(path);
        self
    }

    pub fn include_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.cmake_install_include_dir(path);
        self
    }

    pub fn oldinclude_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.cmake_install_oldinclude_dir(path);
        self
    }

    pub fn dataroot_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.cmake_install_dataroot_dir(path);
        self
    }

    pub fn data_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.cmake_install_data_dir(path);
        self
    }

    pub fn info_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.cmake_install_info_dir(path);
        self
    }

    pub fn locale_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.cmake_install_locale_dir(path);
        self
    }

    pub fn man_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.cmake_install_man_dir(path);
        self
    }

    pub fn doc_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.inner.cmake_install_doc_dir(path);
        self
    }

    pub fn spawn(&mut self) -> io::Result<Child> {
        self.inner.spawn()
    }
}

pub fn configure() -> Configure {
    Configure::new()
}

pub struct Build {
    inner: Inner,
}

impl Build {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let mut inner = Inner::new();

        inner.build(path);

        Self { inner }
    }

    pub fn jobs(&mut self, jobs: usize) -> &mut Self {
        self.inner.jobs(jobs);
        self
    }

    pub fn spawn(&mut self) -> io::Result<Child> {
        self.inner.spawn()
    }
}

pub fn build(path: impl AsRef<Path>) -> Build {
    Build::new(path)
}

pub struct Install {
    inner: Inner,
}

impl Install {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let mut inner = Inner::new();

        inner.install(path);

        Self { inner }
    }

    pub fn spawn(&mut self) -> io::Result<Child> {
        self.inner.spawn()
    }
}

pub fn install(path: impl AsRef<Path>) -> Install {
    Install::new(path)
}
