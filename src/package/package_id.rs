use std::ops::Deref;
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

impl ufmt::uDebug for PackageId {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        ufmt::uDisplay::fmt(self.package_id.deref(), f)
    }
}

impl ufmt::uDisplay for PackageId {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        ufmt::uDisplay::fmt(self.package_id.deref(), f)
    }
}
