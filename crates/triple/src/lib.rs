// Error trait is not in core
// #![no_std]

use core::fmt;
use core::str::FromStr;

pub use crate::arch::Arch;
pub use crate::env::Env;
pub use crate::sys::Sys;

mod arch;
mod env;
mod sys;

const ARMV7L_LINUX_GNU: &str = "armv7l-linux-gnu";
const AARCH64_LINUX_GNU: &str = "aarch64-linux-gnu";
const I686_LINUX_GNU: &str = "i686-linux-gnu";
const X86_64_LINUX_GNU: &str = "x86_64-linux-gnu";

const ARMV7L_LINUX_MUSL: &str = "armv7l-linux-musl";
const AARCH64_LINUX_MUSL: &str = "aarch64-linux-musl";
const I686_LINUX_MUSL: &str = "i686-linux-musl";
const X86_64_LINUX_MUSL: &str = "x86_64-linux-musl";

/// a triple
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Triple {
    arch: Arch,
    sys: Sys,
    env: Env,
}

impl Triple {
    /// returns the host triple (what was used to compile this crate)
    #[inline]
    pub const fn host() -> Self {
        Triple {
            arch: Arch::host(),
            env: Env::host(),
            sys: Sys::host(),
        }
    }

    /// set the architecture
    #[inline]
    pub const fn arch(self, arch: Arch) -> Self {
        let mut this = self;

        this.arch = arch;
        this
    }

    #[inline]
    pub const fn armv7l(self) -> Self {
        self.arch(Arch::armv7l)
    }

    #[inline]
    pub const fn aarch64(self) -> Self {
        self.arch(Arch::aarch64)
    }

    #[inline]
    pub const fn i686(self) -> Self {
        self.arch(Arch::i686)
    }

    #[inline]
    pub const fn x86_64(self) -> Self {
        self.arch(Arch::x86_64)
    }

    /// set the system
    #[inline]
    pub const fn sys(self, sys: Sys) -> Self {
        let mut this = self;

        this.sys = sys;
        this
    }

    #[inline]
    pub const fn linux(self) -> Self {
        self.sys(Sys::Linux)
    }

    /// set the environment
    #[inline]
    pub const fn env(self, env: Env) -> Self {
        let mut this = self;

        this.env = env;
        this
    }

    #[inline]
    pub const fn gnu(self) -> Self {
        self.env(Env::Gnu)
    }

    #[inline]
    pub const fn musl(self) -> Self {
        self.env(Env::Musl)
    }

    /// returns this triple as a tuple
    #[inline]
    pub const fn as_tuple(&self) -> (Arch, Sys, Env) {
        let Triple { arch, sys, env } = *self;

        (arch, sys, env)
    }

    /// returns this triple as a string
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self.as_tuple() {
            (Arch::armv7l, Sys::Linux, Env::Gnu) => ARMV7L_LINUX_GNU,
            (Arch::aarch64, Sys::Linux, Env::Gnu) => AARCH64_LINUX_GNU,
            (Arch::i686, Sys::Linux, Env::Gnu) => I686_LINUX_GNU,
            (Arch::x86_64, Sys::Linux, Env::Gnu) => X86_64_LINUX_GNU,

            (Arch::armv7l, Sys::Linux, Env::Musl) => ARMV7L_LINUX_MUSL,
            (Arch::aarch64, Sys::Linux, Env::Musl) => AARCH64_LINUX_MUSL,
            (Arch::i686, Sys::Linux, Env::Musl) => I686_LINUX_MUSL,
            (Arch::x86_64, Sys::Linux, Env::Musl) => X86_64_LINUX_MUSL,
        }
    }
}

impl fmt::Display for Triple {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TripleParseError {
    InvalidTriple(Box<str>),
}

impl fmt::Display for TripleParseError {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TripleParseError::InvalidTriple(triple) => {
                write!(fmt, "invalid target-triple `{}`", triple.trim())
            }
        }
    }
}

impl std::error::Error for TripleParseError {}

impl FromStr for Triple {
    type Err = TripleParseError;

    #[inline]
    fn from_str(triple: &str) -> Result<Self, Self::Err> {
        let triple = match triple {
            ARMV7L_LINUX_GNU => Triple::host().armv7l().linux().gnu(),
            AARCH64_LINUX_GNU => Triple::host().aarch64().linux().gnu(),
            I686_LINUX_GNU => Triple::host().i686().linux().gnu(),
            X86_64_LINUX_GNU => Triple::host().x86_64().linux().gnu(),

            ARMV7L_LINUX_MUSL => Triple::host().armv7l().linux().musl(),
            AARCH64_LINUX_MUSL => Triple::host().aarch64().linux().musl(),
            I686_LINUX_MUSL => Triple::host().i686().linux().musl(),
            X86_64_LINUX_MUSL => Triple::host().x86_64().linux().musl(),

            _ => return Err(TripleParseError::InvalidTriple(triple.into())),
        };

        Ok(triple)
    }
}
