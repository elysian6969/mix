use std::{error, fmt};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Error {
    pub kind: ErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    UnexpectedEnd(Position),
    UnexpectedChar(Position, char),
    UnexpectedCharAfter(Position, char),
    ExpectedCommaFound(Position, char),
    LeadingZero(Position),
    Overflow(Position),
    EmptySegment(Position),
    IllegalCharacter(Position),
    UnexpectedAfterWildcard,
    ExcessiveComparators,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Position {
    Major,
    Minor,
    Patch,
    Pre,
    Build,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use ErrorKind::*;

        match &self.kind {
            UnexpectedEnd(pos) => {
                write!(fmt, "unexpected end of input while parsing {}", pos)
            }
            UnexpectedChar(pos, ch) => {
                write!(fmt, "unexpected character {:?} while parsing {}", ch, pos,)
            }
            UnexpectedCharAfter(pos, ch) => {
                write!(fmt, "unexpected character {:?} after {}", ch, pos)
            }
            ExpectedCommaFound(pos, ch) => {
                write!(fmt, "expected comma after {}, found {:?}", pos, ch)
            }
            LeadingZero(pos) => {
                write!(fmt, "invalid leading zero in {}", pos)
            }
            Overflow(pos) => {
                write!(fmt, "value of {} exceeds u64::MAX", pos)
            }
            EmptySegment(pos) => {
                write!(fmt, "empty identifier segment in {}", pos)
            }
            IllegalCharacter(pos) => {
                write!(fmt, "unexpected character in {}", pos)
            }
            UnexpectedAfterWildcard => {
                fmt.write_str("unexpected character after wildcard in version req")
            }
            ExcessiveComparators => fmt.write_str("excessive number of version comparators"),
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use Position::*;

        fmt.write_str(match self {
            Major => "major version number",
            Minor => "minor version number",
            Patch => "patch version number",
            Pre => "pre-release identifier",
            Build => "build metadata",
        })
    }
}
