use crate::Shared;
use std::collections::BTreeSet;
use std::fmt;

#[derive(Clone)]
pub struct Set(BTreeSet<Shared>);

impl Set {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }

    pub fn insert(&mut self, package: impl Into<Shared>) {
        let package = package.into();

        self.0.insert(package);
    }

    pub fn remove(&mut self, package: impl Into<Shared>) {
        let package = package.into();

        self.0.remove(&package);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Shared> {
        self.0.iter()
    }
}

impl fmt::Debug for Set {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_set().entries(self.0.iter()).finish()
    }
}
