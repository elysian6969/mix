use std::{error, fmt, io};
use ufmt::derive::uDebug;

#[derive(Debug, uDebug)]
pub enum Exit {
    Code(i32),
    Signal,
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Exit(Exit),
}

impl From<Exit> for Error {
    fn from(error: Exit) -> Self {
        Self::Exit(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl ufmt::uDisplay for Exit {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> std::result::Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        match self {
            Self::Code(code) => ufmt::uwrite!(f, "process exited by code: {}", code),
            Self::Signal => f.write_str("process exited by signal"),
        }
    }
}

impl ufmt::uDisplay for Error {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> std::result::Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        match self {
            Error::Io(io) => ufmt::uwrite!(f, "{}", format!("{io}")),
            Error::Exit(exit) => ufmt::uwrite!(f, "{}", exit),
        }
    }
}

impl fmt::Display for Exit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Code(code) => write!(f, "process exited by code: {code}"),
            Self::Signal => f.write_str("process exited by signal"),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(io) => write!(f, "{io}"),
            Error::Exit(exit) => write!(f, "{exit}"),
        }
    }
}

impl error::Error for Error {}
