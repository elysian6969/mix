pub mod lexer;
pub mod parser;

use self::parser::{Error, Parser};

pub use semver::Version;

pub fn any_version(input: &str) -> Result<Version, Error<'_>> {
    Parser::new(input)?.any_version()
}

pub fn perfect_version(input: &str) -> Result<Version, Error<'_>> {
    Parser::new(input)?.perfect_version()
}
