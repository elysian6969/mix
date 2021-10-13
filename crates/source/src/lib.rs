#![feature(option_result_unwrap_unchecked)]

pub use crate::error::Error;
pub use crate::sources::{Iter, Sources};
use path::{Path, PathBuf};
use std::fmt;
use std::str::FromStr;
use url::Url;

mod error;
mod sources;

#[cfg(feature = "serde")]
mod serde;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Kind {
    Github,
    Gitlab,
    Gnu,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Source {
    /// source tag
    kind: Kind,

    /// offset of `/` if the tag requires.
    split: usize,

    /// cached serialization
    serialization: String,

    /// cached url
    url: Url,
}

impl Source {
    fn new_path(kind: Kind, prefix: &str, path: &str) -> Self {
        let serialization = path.into();
        let split = 0;
        let url = build_url_path(prefix, path);

        Source {
            kind,
            split,
            serialization,
            url,
        }
    }

    fn new_repo(kind: Kind, prefix: &str, user: &str, repository: &str) -> Self {
        let serialization = serialize_user_repository(user, repository);
        let split = user.len();
        let url = build_url_repo(prefix, user, repository);

        Source {
            kind,
            split,
            serialization,
            url,
        }
    }

    pub fn github(user: &str, repository: &str) -> Self {
        Self::new_repo(Kind::Github, "https://github.com/", user, repository)
    }

    pub fn gitlab(user: &str, repository: &str) -> Self {
        Self::new_repo(Kind::Gitlab, "https://gitlab.com/", user, repository)
    }

    pub fn gnu(path: &str) -> Self {
        Self::new_path(Kind::Gnu, "http://ftp.gnu.org/gnu/", path)
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn user(&self) -> Option<&str> {
        use Kind::*;

        match self.kind {
            // SAFETY: `split` is always a valid index within `serialization`.
            Github | Gitlab => Some(unsafe { self.serialization.get_unchecked(..self.split) }),
            _ => None,
        }
    }

    pub fn repository(&self) -> Option<&str> {
        use Kind::*;

        match self.kind {
            // SAFETY: `split` is always a valid index within `serialization`.
            Github | Gitlab => Some(unsafe { self.serialization.get_unchecked(self.split..) }),
            _ => None,
        }
    }

    pub fn path(&self) -> Option<&str> {
        use Kind::*;

        match self.kind {
            Gnu => Some(self.serialization.as_str()),
            _ => None,
        }
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn cache(&self, prefix: impl AsRef<Path>) -> PathBuf {
        use Kind::*;

        let prefix = prefix.as_ref();

        match self.kind {
            Github => prefix.join("github").join(&self.serialization),
            Gitlab => prefix.join("gitlab").join(&self.serialization),
            Gnu => prefix.join("gnu").join(&self.serialization),
        }
    }
}

#[inline]
fn serialize_user_repository(user: &str, repository: &str) -> String {
    let len = user.len().saturating_add(repository.len());
    let mut string = String::with_capacity(len);

    string.push_str(user);
    string.push('/');
    string.push_str(repository);
    string
}

#[inline]
fn build_url_path(prefix: &str, path: &str) -> Url {
    let len = prefix.len().saturating_add(path.len());

    let mut string = String::with_capacity(len);

    string.push_str(prefix);
    string.push_str(path);

    // SAFETY: This should always be valid.
    unsafe { Url::parse(string.as_str()).unwrap_unchecked() }
}

#[inline]
fn build_url_repo(prefix: &str, user: &str, repository: &str) -> Url {
    let len = prefix
        .len()
        .saturating_add(user.len())
        .saturating_add(repository.len());

    let mut string = String::with_capacity(len);

    string.push_str(prefix);
    string.push_str(user);
    string.push('/');
    string.push_str(repository);

    // SAFETY: This should always be valid.
    unsafe { Url::parse(string.as_str()).unwrap_unchecked() }
}

impl FromStr for Source {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // If there is no colon, set to start of input.
        let colon = input.find(':').unwrap_or(0);

        // SAFETY: `colon` is guarenteed to be a valid position within `input`.
        let scheme = unsafe { input.get_unchecked(..colon) };

        match scheme.len() {
            3 => match scheme {
                "gnu" => {
                    // SAFETY: `colon` is guarenteed to be a valid position within `input`.
                    let repository =
                        unsafe { input.get_unchecked(colon.saturating_add(1).min(input.len())..) };
                    let repository = repository.trim();

                    if repository.is_empty() {
                        return Err(Error::ExpectedRepository);
                    }

                    Ok(Source::gnu(repository))
                }
                _ => Err(Error::UnknownScheme),
            },
            6 => match scheme {
                "github" => {
                    // SAFETY: `colon` is guarenteed to be a valid position within `input`.
                    let input =
                        unsafe { input.get_unchecked(colon.saturating_add(1).min(input.len())..) };

                    // If there is no slash, set to start of input.
                    let slash = input.find('/').unwrap_or(0);

                    // SAFETY: `slash` is guarenteed to be a valid position within `input`.
                    let user = unsafe { input.get_unchecked(..slash) };
                    let user = user.trim();

                    if user.is_empty() {
                        return Err(Error::ExpectedUser);
                    }

                    // SAFETY: `slash` is guarenteed to be a valid position within `input`.
                    let repository =
                        unsafe { input.get_unchecked(slash.saturating_add(1).min(input.len())..) };
                    let repository = repository.trim();

                    if repository.is_empty() {
                        return Err(Error::ExpectedRepository);
                    }

                    Ok(Source::github(user.into(), repository.into()))
                }
                "gitlab" => {
                    // SAFETY: `colon` is guarenteed to be a valid position within `input`.
                    let input =
                        unsafe { input.get_unchecked(colon.saturating_add(1).min(input.len())..) };

                    // If there is no slash, set to start of input.
                    let slash = input.find('/').unwrap_or(0);

                    // SAFETY: `slash` is guarenteed to be a valid position within `input`.
                    let user = unsafe { input.get_unchecked(..slash) };
                    let user = user.trim();

                    if user.is_empty() {
                        return Err(Error::ExpectedUser);
                    }

                    // SAFETY: `slash` is guarenteed to be a valid position within `input`.
                    let repository =
                        unsafe { input.get_unchecked(slash.saturating_add(1).min(input.len())..) };
                    let repository = repository.trim();

                    if repository.is_empty() {
                        return Err(Error::ExpectedRepository);
                    }

                    Ok(Source::gitlab(user.into(), repository.into()))
                }
                _ => Err(Error::UnknownScheme),
            },
            _ => Err(Error::UnknownScheme),
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use Kind::*;

        match self.kind {
            Github => {
                fmt.write_str("github:")?;
                fmt.write_str(&self.serialization)?;
            }
            Gitlab => {
                fmt.write_str("gitlab:")?;
                fmt.write_str(&self.serialization)?;
            }
            Gnu => {
                fmt.write_str("gnu:")?;
                fmt.write_str(&self.serialization)?;
            }
        }

        Ok(())
    }
}
