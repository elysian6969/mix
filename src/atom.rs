use semver::VersionReq;
use std::str::FromStr;

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct Atom {
    pub name: String,
    pub version: VersionReq,
}

impl Atom {
    pub fn parse(atom: &str) -> crate::Result<Self> {
        Atom::from_str(atom)
    }
}

impl FromStr for Atom {
    type Err = crate::Error;

    fn from_str(atom: &str) -> crate::Result<Self> {
        match atom.split_once(':') {
            Some((name, version)) => Ok(Self {
                name: name.to_string(),
                version: VersionReq::parse(version)?,
            }),
            None => Ok(Self {
                name: atom.to_string(),
                version: VersionReq::any(),
            }),
        }
    }
}
