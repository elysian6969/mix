use command_extra::{Child, Stdio};
use path::Path;
use std::ffi::{OsStr, OsString};
use std::io;

pub struct Command {
    command: command_extra::Command,
}

const ROOT: &str = "/";

pub mod program {
    pub(crate) const CARGO: &str = "cargo";
    pub(crate) const CMAKE: &str = "cmake";
    pub(crate) const MAKE: &str = "make";
    pub(crate) const MESON: &str = "meson";
    pub(crate) const SH: &str = "sh";
}

mod env {
    pub(crate) const CC: &str = "CC";
    pub(crate) const CFLAGS: &str = "CFLAGS";

    pub(crate) const CXX: &str = "CXX";
    pub(crate) const CXXFLAGS: &str = "CXXFLAGS";

    pub(crate) const HOME: &str = "HOME";

    pub(crate) const LANG: &str = "LANG";

    pub(crate) const LD: &str = "LD";
    pub(crate) const LDFLAGS: &str = "LDFLAGS";

    pub(crate) const PATH: &str = "PATH";
    pub(crate) const PATH_SEPERATOR: &str = ":";

    pub(crate) const PS1: &str = "PS1";
}

impl Command {
    pub fn new<P>(program: P) -> Self
    where
        P: AsRef<OsStr>,
    {
        let command = command_extra::Command::new(program);

        Self { command }
    }

    pub fn sh() -> Self {
        Self::new(program::SH)
    }

    pub fn make() -> Self {
        Self::new(program::MAKE)
    }

    pub fn cargo() -> Self {
        Self::new(program::CARGO)
    }

    pub fn arg<A>(&mut self, arg: A) -> &mut Self
    where
        A: AsRef<OsStr>,
    {
        self.command.arg(arg);
        self
    }

    pub fn args<I, A>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = A>,
        A: AsRef<OsStr>,
    {
        self.command.args(args);
        self
    }

    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.command.env(key, val);
        self
    }

    pub fn envs<I, K, V>(&mut self, envs: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.command.envs(envs);
        self
    }

    pub fn env_clear(&mut self) -> &mut Self {
        self.command.env_clear();
        self
    }

    pub fn env_remove<K>(&mut self, key: K) -> &mut Self
    where
        K: AsRef<OsStr>,
    {
        self.command.env_remove(key);
        self
    }

    pub fn c_compiler<C>(&mut self, compiler: C) -> &mut Self
    where
        C: AsRef<OsStr>,
    {
        self.command.env(env::CC, compiler);
        self
    }

    pub fn c_flags<F>(&mut self, flags: F) -> &mut Self
    where
        F: AsRef<OsStr>,
    {
        self.command.env(env::CFLAGS, flags);
        self
    }

    pub fn cxx_compiler<C>(&mut self, compiler: C) -> &mut Self
    where
        C: AsRef<OsStr>,
    {
        self.command.env(env::CXX, compiler);
        self
    }

    pub fn cxx_flags<F>(&mut self, flags: F) -> &mut Self
    where
        F: AsRef<OsStr>,
    {
        self.command.env(env::CXXFLAGS, flags);
        self
    }

    pub fn current_dir<D>(&mut self, dir: D) -> &mut Self
    where
        D: AsRef<Path>,
    {
        self.command.current_dir(dir);
        self
    }

    pub fn home_dir<D>(&mut self, dir: D) -> &mut Self
    where
        D: AsRef<Path>,
    {
        self.command.env(env::HOME, dir.as_ref());
        self
    }

    pub fn ps1<P>(&mut self, ps1: P) -> &mut Self
    where
        P: AsRef<OsStr>,
    {
        self.command.env(env::PS1, ps1);
        self
    }

    pub fn lang<L>(&mut self, lang: L) -> &mut Self
    where
        L: AsRef<OsStr>,
    {
        self.command.env(env::LANG, lang);
        self
    }

    pub fn paths<I, P>(&mut self, paths: I) -> &mut Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let paths = paths
            .into_iter()
            // SAFETY: silence rustc, silence...
            .map(|path| unsafe { std::mem::transmute::<_, &OsStr>(path.as_ref()) })
            .intersperse(unsafe { std::mem::transmute::<_, &OsStr>(env::PATH_SEPERATOR) })
            .collect::<OsString>();

        self.command.env(env::PATH, paths);
        self
    }

    pub fn linker<L>(&mut self, linker: L) -> &mut Self
    where
        L: AsRef<OsStr>,
    {
        self.command.env(env::LD, linker);
        self
    }

    pub fn linker_flags<F>(&mut self, flags: F) -> &mut Self
    where
        F: AsRef<OsStr>,
    {
        self.command.env(env::LDFLAGS, flags);
        self
    }

    pub fn stdin<S>(&mut self, stdin: S) -> &mut Self
    where
        S: Into<Stdio>,
    {
        self.command.stdin(stdin);
        self
    }

    pub fn stdout<S>(&mut self, stdout: S) -> &mut Self
    where
        S: Into<Stdio>,
    {
        self.command.stdout(stdout);
        self
    }

    pub fn stderr<S>(&mut self, stderr: S) -> &mut Self
    where
        S: Into<Stdio>,
    {
        self.command.stderr(stderr);
        self
    }

    pub async fn spawn(&mut self) -> io::Result<Child> {
        self.command.spawn().await
    }
}
