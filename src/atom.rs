use semver::VersionReq;
use std::str::FromStr;
use ufmt::derive::uDebug;

#[derive(Hash, Eq, PartialEq)]
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

impl ufmt::uDebug for Atom {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        struct Version<'version>(&'version VersionReq);

        impl<'version> ufmt::uDebug for Version<'version> {
            fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
            where
                W: ufmt::uWrite + ?Sized,
            {
                let buf = format!("{:?}", self.0);

                ufmt::uwrite!(f, "{}", buf)
            }
        }

        f.debug_struct("Atom")?
            .field("name", &self.name)?
            .field("repositories", &Version(&self.version))?
            .finish()
    }
}

impl ufmt::uDisplay for Atom {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        struct Version<'version>(&'version VersionReq);

        impl<'version> ufmt::uDisplay for Version<'version> {
            fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
            where
                W: ufmt::uWrite + ?Sized,
            {
                let buf = format!("{}", self.0);

                ufmt::uwrite!(f, "{}", buf)
            }
        }

        ufmt::uwrite!(f, "{}:{}", self.name, Version(&self.version))
    }
}
