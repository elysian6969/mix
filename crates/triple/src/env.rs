use core::fmt;

const GNU: &str = "gnu";
const MUSL: &str = "musl";

/// an environment
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Env {
    Gnu,
    Musl,
}

impl Env {
    /// returns the host environment (what was used to compile this crate)
    #[inline]
    pub const fn host() -> Env {
        #[cfg(target_env = "gnu")]
        const HOST: Env = Env::Gnu;

        #[cfg(target_env = "musl")]
        const HOST: Env = Env::Musl;

        HOST
    }

    /// returns this environment as a string
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Env::Gnu => GNU,
            Env::Musl => MUSL,
        }
    }
}

impl fmt::Debug for Env {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Display for Env {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
