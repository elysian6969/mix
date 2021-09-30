use core::fmt;

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum EnvRepr {
    Glibc,
    Musl,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Env {
    repr: EnvRepr,
}

impl Env {
    const fn new(repr: EnvRepr) -> Self {
        Self { repr }
    }

    pub const fn glibc() -> Self {
        Self::new(EnvRepr::Glibc)
    }

    pub const fn musl() -> Self {
        Self::new(EnvRepr::Musl)
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            const { Env::glibc() } => "glibc",
            const { Env::musl() } => "musl",
        }
    }
}

impl fmt::Debug for Env {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Display for Env {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
