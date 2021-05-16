use std::io;
use ufmt::derive::uDebug;

#[derive(uDebug)]
pub enum Exit {
    Code(i32),
    Signal,
}

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
    fn fmt<W>(&self, fmt: &mut ufmt::Formatter<'_, W>) -> std::result::Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        match *self {
            Self::Code(code) => ufmt::uwrite!(fmt, "process exited by code: {}", code),
            Self::Signal => fmt.write_str("process exited by signal"),
        }
    }
}

impl ufmt::uDisplay for Error {
    fn fmt<W>(&self, fmt: &mut ufmt::Formatter<'_, W>) -> std::result::Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        match self {
            Error::Io(io) => ufmt::uwrite!(fmt, "{}", format!("{io}")),
            Error::Exit(exit) => ufmt::uwrite!(fmt, "{}", exit),
        }
    }
}
