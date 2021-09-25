use crate::{Error, ErrorKind};

pub fn validate(id: &str, kind: ErrorKind) -> Result<(), Error> {
    if let Some(index) = id.chars().position(|c| !c.is_ascii_alphanumeric()) {
        Err(Error::UnexpectedCharacter { index, kind })
    } else {
        Ok(())
    }
}
