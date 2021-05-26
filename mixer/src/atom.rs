pub mod lexer;
pub mod parser;

use parser::Parser;
use semver::VersionReq;

#[derive(Hash, Eq, PartialEq)]
pub struct Atom {
    pub group: Option<String>,
    pub package: String,
    pub version: VersionReq,
}

impl Atom {
    pub fn parse(input: &str) -> crate::Result<Self> {
        let source = Parser::new(input)
            .map_err(|error| ufmt::uformat!("TODO: {:?}", error).expect("infallible"))?
            .parse()
            .map_err(|error| ufmt::uformat!("TODO: {:?}", error).expect("infallible"))?;

        Ok(source)
    }
}

struct Version<'version>(&'version VersionReq);

impl<'version> ufmt::uDebug for Version<'version> {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        let buffer = format!("{:?}", self.0);

        ufmt::uwrite!(f, "{}", buffer)
    }
}

impl<'version> ufmt::uDisplay for Version<'version> {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        let buffer = self.0.to_string();

        ufmt::uwrite!(f, "{}", buffer)
    }
}

impl ufmt::uDebug for Atom {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        f.debug_struct("Atom")?
            .field("group", &self.group)?
            .field("package", &self.package)?
            .field("version", &Version(&self.version))?
            .finish()
    }
}

impl ufmt::uDisplay for Atom {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        if let Some(ref group) = self.group.as_ref() {
            ufmt::uwrite!(f, "{}/", group)?;
        }

        ufmt::uwrite!(f, "{}:{}", self.package, Version(&self.version))
    }
}
