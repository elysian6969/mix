use core::fmt;

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum ArchRepr {
    Armv7l,
    AArch64,
    I686,
    X86_64,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Arch {
    repr: ArchRepr,
}

impl Arch {
    const fn new(repr: ArchRepr) -> Self {
        Self { repr }
    }

    pub const fn armv7l() -> Self {
        Self::new(ArchRepr::Armv7l)
    }

    pub const fn aarch64() -> Self {
        Self::new(ArchRepr::AArch64)
    }

    pub const fn i686() -> Self {
        Self::new(ArchRepr::I686)
    }

    pub const fn x86_64() -> Self {
        Self::new(ArchRepr::X86_64)
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            const { Arch::armv7l() } => "armv7l",
            const { Arch::aarch64() } => "aarch64",
            const { Arch::i686() } => "i686",
            const { Arch::x86_64() } => "x86_64",
        }
    }
}

impl fmt::Debug for Arch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
