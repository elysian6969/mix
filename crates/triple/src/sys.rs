use core::fmt;

/// a system
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Sys {
    Linux,
}

impl Sys {
    /// returns the host system (what was used to compile this crate)
    #[inline]
    pub const fn host() -> Self {
        #[cfg(target_os = "linux")]
        const HOST: Sys = Sys::Linux;

        HOST
    }

    /// returns this system as a string
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Sys::Linux => "linux",
        }
    }
}

impl fmt::Debug for Sys {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Display for Sys {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
