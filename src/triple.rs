use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Triple {
    pub architecture: &'static str,
    pub vendor: &'static str,
    pub system: &'static str,
    pub interface: &'static str,
}

impl Triple {
    /// Create new triple
    pub const fn new(architecture: &'static str) -> Self {
        Self {
            architecture,
            vendor: "",
            system: "",
            interface: "",
        }
    }

    /// Create new AArch64 triple
    pub const fn aarch64() -> Self {
        Self::new("aarch64")
    }

    /// Create new x86_64 triple
    pub const fn x86_64() -> Self {
        Self::new("x86_64")
    }

    /// Set vendor
    pub const fn vendor(mut self, vendor: &'static str) -> Self {
        self.vendor = vendor;
        self
    }

    /// Set system
    pub const fn system(mut self, system: &'static str) -> Self {
        self.system = system;
        self
    }

    /// Set system to Linux
    pub const fn linux(self) -> Self {
        self.system("linux")
    }

    /// Set ABI
    pub const fn interface(mut self, interface: &'static str) -> Self {
        self.interface = interface;
        self
    }

    /// Set ABI to GNU
    pub const fn gnu(self) -> Self {
        self.interface("gnu")
    }

    /// Set ABI to musl
    pub const fn musl(self) -> Self {
        self.interface("musl")
    }

    #[cfg(target_arch = "aarch64")]
    pub fn default() -> Self {
        Triple::aarch64().linux().gnu()
    }

    #[cfg(target_arch = "x86_64")]
    pub fn default() -> Self {
        Triple::x86_64().linux().gnu()
    }
}

impl fmt::Display for Triple {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn unknown_if_empty(string: &str) -> &str {
            if string.is_empty() {
                "unknown"
            } else {
                string
            }
        }

        write!(
            f,
            "{}-{}-{}-{}",
            unknown_if_empty(self.architecture),
            unknown_if_empty(self.vendor),
            unknown_if_empty(self.system),
            unknown_if_empty(self.interface),
        )
    }
}
