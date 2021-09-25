use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    UnexpectedCharacter { index: usize, kind: ErrorKind },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    Package,
    Repository,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        match &self {
            UnexpectedCharacter {
                index,
                kind: ErrorKind::Package,
            } => {
                fmt.write_fmt(format_args!(
                    "unexpected character in package id at {}",
                    index
                ))?;
            }
            UnexpectedCharacter {
                index,
                kind: ErrorKind::Repository,
            } => {
                fmt.write_fmt(format_args!(
                    "unexpected character in repository id at {}",
                    index
                ))?;
            }
        }

        Ok(())
    }
}
