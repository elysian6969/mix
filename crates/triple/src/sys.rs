use core::fmt;

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum SysRepr {
    Linux,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Sys {
    repr: SysRepr,
}

impl Sys {
    const fn new(repr: SysRepr) -> Self {
        Self { repr }
    }

    pub const fn linux() -> Self {
        Self::new(SysRepr::Linux)
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            const { Sys::linux() } => "linux",
        }
    }
}

impl fmt::Debug for Sys {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Display for Sys {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
