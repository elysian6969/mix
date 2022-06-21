use core::fmt;

const ARMV7L: &str = "armv7l";
const AARCH64: &str = "aarch64";
const I686: &str = "i686";
const X86_64: &str = "x86_64";

/// an architecture
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Arch {
    armv7l,
    aarch64,
    i686,
    x86_64,
}

impl Arch {
    /// returns the host architecture (what was used to compile this crate)
    #[inline]
    pub const fn host() -> Arch {
        #[cfg(target_arch = "armv7l")]
        const HOST: Arch = Arch::armv7l;

        #[cfg(target_arch = "aarch64")]
        const HOST: Arch = Arch::aarch64;

        #[cfg(target_arch = "i686")]
        const HOST: Arch = Arch::i686;

        #[cfg(target_arch = "x86_64")]
        const HOST: Arch = Arch::x86_64;

        HOST
    }

    /// returns this architecture as a string
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Arch::armv7l => ARMV7L,
            Arch::aarch64 => AARCH64,
            Arch::i686 => I686,
            Arch::x86_64 => X86_64,
        }
    }
}

impl fmt::Debug for Arch {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Display for Arch {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
