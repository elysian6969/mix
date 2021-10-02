#![allow(incomplete_features)]
#![feature(const_trait_impl)]
#![feature(inline_const)]
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

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Triple {
    arch: Arch,
    sys: Sys,
    env: Env,
}

impl Triple {
    const fn new(arch: Arch) -> Self {
        Self {
            arch,
            sys: Sys::linux(),
            env: Env::glibc(),
        }
    }

    pub const fn armv7l() -> Self {
        Self::new(Arch::armv7l())
    }

    pub const fn aarch64() -> Self {
        Self::new(Arch::aarch64())
    }

    pub const fn i686() -> Self {
        Self::new(Arch::i686())
    }

    pub const fn x86_64() -> Self {
        Self::new(Arch::x86_64())
    }

    pub const fn sys(self, sys: Sys) -> Self {
        let mut this = self;
        this.sys = sys;
        this
    }

    pub const fn linux(self) -> Self {
        self.sys(Sys::linux());
        self
    }

    pub const fn env(self, env: Env) -> Self {
        let mut this = self;
        this.env = env;
        this
    }

    pub const fn glibc(self) -> Self {
        self.env(Env::glibc())
    }

    pub const fn musl(self) -> Self {
        self.env(Env::musl())
    }

    pub const fn host() -> Self {
        // glibc

        #[cfg(all(target_arch = "armv7l", target_os = "linux", target_env = "gnu"))]
        const HOST: Triple = Triple::armv7l();

        #[cfg(all(target_arch = "aarch64", target_os = "linux", target_env = "gnu"))]
        const HOST: Triple = Triple::aarch64();

        #[cfg(all(target_arch = "i686", target_os = "linux", target_env = "gnu"))]
        const HOST: Triple = Triple::i686();

        #[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
        const HOST: Triple = Triple::x86_64();

        // musl

        #[cfg(all(target_arch = "armv7l", target_os = "linux", target_env = "musl"))]
        const HOST: Triple = Triple::armv7l().musl();

        #[cfg(all(target_arch = "aarch64", target_os = "linux", target_env = "musl"))]
        const HOST: Triple = Triple::aarch64().musl();

        #[cfg(all(target_arch = "i686", target_os = "linux", target_env = "musl"))]
        const HOST: Triple = Triple::i686().musl();

        #[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "musl"))]
        const HOST: Triple = Triple::x86_64().musl();

        HOST
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            const { Triple::armv7l() } => "armv7l-linux-gnu",
            const { Triple::aarch64() } => "aarch64-linux-gnu",
            const { Triple::i686() } => "i686-linux-gnu",
            const { Triple::x86_64() } => "x86_64-linux-gnu",

            const { Triple::armv7l().musl() } => "armv7l-linux-musl",
            const { Triple::aarch64().musl() } => "aarch64-linux-musl",
            const { Triple::i686().musl() } => "i686-linux-musl",
            const { Triple::x86_64().musl() } => "x86_64-linux-musl",
        }
    }

    pub const fn as_gnu_str(&self) -> &'static str {
        match self {
            const { Triple::armv7l() } => "armv7l-unknown-linux-gnu",
            const { Triple::aarch64() } => "aarch64-unknown-linux-gnu",
            const { Triple::i686() } => "i686-pc-linux-gnu",
            const { Triple::x86_64() } => "x86_64-pc-linux-gnu",

            const { Triple::armv7l().musl() } => "armv7l-unknown-linux-musl",
            const { Triple::aarch64().musl() } => "aarch64-unknown-linux-musl",
            const { Triple::i686().musl() } => "i686-pc-linux-musl",
            const { Triple::x86_64().musl() } => "x86_64-pc-linux-musl",
        }
    }

    pub const fn as_llvm_str(&self) -> &'static str {
        match self {
            const { Triple::armv7l() } => "armv7l-unknown-linux-gnu",
            const { Triple::aarch64() } => "aarch64-unknown-linux-gnu",
            const { Triple::i686() } => "i686-unknown-linux-gnu",
            const { Triple::x86_64() } => "x86_64-unknown-linux-gnu",

            const { Triple::armv7l().musl() } => "armv7l-unknown-linux-musl",
            const { Triple::aarch64().musl() } => "aarch64-unknown-linux-musl",
            const { Triple::i686().musl() } => "i686-unknown-linux-musl",
            const { Triple::x86_64().musl() } => "x86_64-unknown-linux-musl",
        }
    }

    pub const fn arch_str(&self) -> &'static str {
        match self {
            const { Triple::armv7l() } => "armv7l",
            const { Triple::aarch64() } => "aarch64",
            const { Triple::i686() } => "i686",
            const { Triple::x86_64() } => "x86_64",

            const { Triple::armv7l().musl() } => "armv7l",
            const { Triple::aarch64().musl() } => "aarch64",
            const { Triple::i686().musl() } => "i686",
            const { Triple::x86_64().musl() } => "x86_64",
        }
    }
}

impl fmt::Display for Triple {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TripleParseError {
    UnknownTriple,
}

impl fmt::Display for TripleParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TripleParseError::UnknownTriple => write!(fmt, "Unknown target-triple."),
        }
    }
}

impl std::error::Error for TripleParseError {}

// TODO: Not this.
const ARMV7L_LINUX_GNU: Triple = Triple::armv7l();
const AARCH64_LINUX_GNU: Triple = Triple::aarch64();
const I686_LINUX_GNU: Triple = Triple::i686();
const X86_64_LINUX_GNU: Triple = Triple::x86_64();

const ARMV7L_LINUX_MUSL: Triple = Triple::armv7l().musl();
const AARCH64_LINUX_MUSL: Triple = Triple::aarch64().musl();
const I686_LINUX_MUSL: Triple = Triple::i686().musl();
const X86_64_LINUX_MUSL: Triple = Triple::x86_64().musl();

const ARMV7L_LINUX_GNU_STR: &str = ARMV7L_LINUX_GNU.as_str();
const AARCH64_LINUX_GNU_STR: &str = AARCH64_LINUX_GNU.as_str();
const I686_LINUX_GNU_STR: &str = I686_LINUX_GNU.as_str();
const X86_64_LINUX_GNU_STR: &str = X86_64_LINUX_GNU.as_str();

const ARMV7L_LINUX_MUSL_STR: &str = ARMV7L_LINUX_MUSL.as_str();
const AARCH64_LINUX_MUSL_STR: &str = AARCH64_LINUX_MUSL.as_str();
const I686_LINUX_MUSL_STR: &str = I686_LINUX_MUSL.as_str();
const X86_64_LINUX_MUSL_STR: &str = X86_64_LINUX_MUSL.as_str();

impl FromStr for Triple {
    type Err = TripleParseError;

    fn from_str(triple: &str) -> Result<Self, Self::Err> {
        let triple = match triple {
            ARMV7L_LINUX_GNU_STR => ARMV7L_LINUX_GNU,
            AARCH64_LINUX_GNU_STR => AARCH64_LINUX_GNU,
            I686_LINUX_GNU_STR => I686_LINUX_GNU,
            X86_64_LINUX_GNU_STR => X86_64_LINUX_GNU,

            ARMV7L_LINUX_MUSL_STR => ARMV7L_LINUX_MUSL,
            AARCH64_LINUX_MUSL_STR => AARCH64_LINUX_MUSL,
            I686_LINUX_MUSL_STR => I686_LINUX_MUSL,
            X86_64_LINUX_MUSL_STR => X86_64_LINUX_MUSL,

            _ => return Err(TripleParseError::UnknownTriple),
        };

        Ok(triple)
    }
}
