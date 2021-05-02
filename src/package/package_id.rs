use std::fmt;
use std::sync::Arc;

#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct PackageId {
    package_id: Arc<String>,
}

impl PackageId {
    pub fn new(package_id: impl Into<String>) -> Self {
        Self {
            package_id: Arc::new(package_id.into()),
        }
    }

    pub fn as_str(&self) -> &str {
        self.package_id.as_str()
    }
}

impl fmt::Debug for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.package_id, f)
    }
}

impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.package_id, f)
    }
}
