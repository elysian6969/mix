use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    UnknownScheme,
    ExpectedRepository,
    ExpectedUser,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        match &self {
            UnknownScheme => fmt.write_str("unknown scheme")?,
            ExpectedRepository => fmt.write_str("expected repository")?,
            ExpectedUser => fmt.write_str("expected user")?,
        }

        Ok(())
    }
}

impl error::Error for Error {}
