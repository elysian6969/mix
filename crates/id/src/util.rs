use crate::{Error, ErrorKind};

pub fn validate(id: &str, kind: ErrorKind) -> Result<(), Error> {
    if let Some(index) = id.chars().position(|c| !is_valid(c)) {
        Err(Error::UnexpectedCharacter { index, kind })
    } else {
        Ok(())
    }
}

fn is_valid(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-'
}
