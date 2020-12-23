#[derive(Clone, Copy, Debug)]
pub struct Triple<'triple> {
    arch: &'triple str,
    vendor: Option<&'triple str>,
    sys: Option<&'triple str>,
    abi: Option<&'triple str>,
}

impl<'triple> Triple<'triple> {
    /// Create new target triple with the specified architecture
    pub const fn new(arch: &'triple str) -> Self {
        Self {
            arch,
            vendor: None,
            sys: None,
            abi: None,
        }
    }

    /// Specify the vendor
    pub const fn vendor(mut self, vendor: &'triple str) -> Self {
        self.vendor = Some(vendor);
        self
    }

    /// Specify the system
    pub const fn sys(mut self, sys: &'triple str) -> Self {
        self.sys = Some(sys);
        self
    }

    /// Specify the ABI
    pub const fn abi(mut self, abi: &'triple str) -> Self {
        self.abi = Some(abi);
        self
    }

    /// Set the system to linux
    pub const fn linux(self) -> Self {
        self.sys("linux")
    }

    /// Set the ABI to GNU
    pub const fn gnu(self) -> Self {
        self.abi("gnu")
    }

    /// Set the ABI to musl
    pub const fn musl(self) -> Self {
        self.abi("musl")
    }

    pub fn to_string(&self) -> Option<String> {
        match &self {
            Triple {
                arch,
                vendor: Some(vendor),
                sys: Some(sys),
                abi: Some(abi),
            } => Some(format!("{}-{}-{}-{}", arch, vendor, sys, abi)),
            Triple {
                arch,
                vendor: None,
                sys: Some(sys),
                abi: Some(abi),
            } => Some(format!("{}-unknown-{}-{}", arch, sys, abi)),
            _ => None,
        }
    }
}

/// aarch64-unknown-linux-gnu
pub const AARCH64_UNKNOWN_LINUX_GNU: Triple<'static> = Triple::new("aarch64").linux().gnu();
/// aarch64-unknown-linux-musl
pub const AARCH64_UNKNOWN_LINUX_MUSL: Triple<'static> = Triple::new("aarch64").linux().musl();

/// x86_64-unknown-linux-gnu
pub const X86_64_UNKNOWN_LINUX_GNU: Triple<'static> = Triple::new("x86_64").linux().gnu();
/// x86_64-unknown-linux-musl
pub const X86_64_UNKNOWN_LINUX_MUSL: Triple<'static> = Triple::new("x86_64").linux().musl();
