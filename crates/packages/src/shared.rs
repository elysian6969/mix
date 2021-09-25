use metadata::Package;
use std::rc::Rc;
use std::{fmt, ops};

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Shared(Rc<Package>);

impl From<Package> for Shared {
    fn from(package: Package) -> Self {
        Self(Rc::new(package))
    }
}

impl From<Rc<Package>> for Shared {
    fn from(package: Rc<Package>) -> Self {
        Self(package)
    }
}

impl fmt::Debug for Shared {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "{}/{}:{}",
            self.group(),
            self.package(),
            self.version()
        )
    }
}

impl ops::Deref for Shared {
    type Target = Package;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
