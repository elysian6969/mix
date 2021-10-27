#![feature(map_first_last)]
#![feature(option_result_unwrap_unchecked)]
#![feature(str_split_as_str)]

pub use crate::error::Error;
pub use crate::sources::{Iter, Sources};
use mix_shell::{async_trait, write, AsyncDisplay, Shell};
use mix_version::Version;
use path::{Path, PathBuf};
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;
use url::Url;

mod github;
mod gitlab;

mod error;
mod sources;

pub mod versions;

//#[cfg(feature = "serde")]
mod serde;

pub(crate) type Error2 = Box<dyn std::error::Error + Send + Sync + 'static>;
pub(crate) type Result<T, E = Error2> = std::result::Result<T, E>;

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
            Github | Gitlab => Some(unsafe {
                self.serialization
                    .get_unchecked(self.split.saturating_add(1).min(self.serialization.len())..)
            }),
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

    pub async fn update(&self, config: &mix_config::Config) -> Result<()> {
        match self.kind() {
            Kind::Github => unsafe {
                let url = format!(
                    "{base}/repos/{user}/{repo}/tags",
                    base = "https://api.github.com",
                    user = self.user().unwrap_unchecked(),
                    repo = self.repository().unwrap_unchecked()
                );

                let dir = self.cache(config.cache_prefix());
                let tags = dir.join("tags.json");
                let _ = dir.create_dir_all_async().await;

                config.download_file(tags, url).await?;
            },
            _ => {}
        }

        Ok(())
    }

    pub async fn versions(&self, config: &mix_config::Config) -> Result<versions::Versions> {
        let dir = self.cache(config.cache_prefix());
        let tags = dir.join("tags.json");

        let versions = match self.kind() {
            Kind::Github => {
                let slice = tags.read_async().await?;
                let tags: Vec<github::Tag> = serde_json::from_slice(&slice)?;

                tags.into_iter()
                    .flat_map(|tag| {
                        let version = Version::parse_anything(&tag.name?);
                        let url = Url::parse(&tag.tarball_url?).ok()?;
                        let path = dir.join(format!("v{}.tar.gz", &version));
                        let file = self::versions::Entry {
                            path,
                            url,
                            version: version.clone(),
                        };

                        Some((version, file))
                    })
                    .collect::<BTreeMap<_, _>>()
            }
            _ => BTreeMap::new(),
        };

        Ok(versions::Versions { versions })
    }
}

#[async_trait(?Send)]
impl AsyncDisplay<Shell> for Source {
    async fn fmt(&self, fmt: &Shell) -> Result<()> {
        use Kind::*;

        match self.kind {
            Github => {
                // SAFETY: unwrapping user and repository is safe for this kind.
                let (user, repository) = unsafe {
                    let user = self.user().unwrap_unchecked();
                    let repository = self.repository().unwrap_unchecked();

                    (user, repository)
                };

                write!(
                    fmt,
                    "{}{}{}{}{}",
                    fmt.theme().arguments_paint("github"),
                    fmt.theme().seperator_paint(':'),
                    fmt.theme().arguments_paint(user),
                    fmt.theme().seperator_paint('/'),
                    fmt.theme().arguments_paint(repository),
                )?;
            }
            Gitlab => {
                // SAFETY: unwrapping user and repository is safe for this kind.
                let (user, repository) = unsafe {
                    let user = self.user().unwrap_unchecked();
                    let repository = self.repository().unwrap_unchecked();

                    (user, repository)
                };

                write!(
                    fmt,
                    "{}{}{}{}{}",
                    fmt.theme().arguments_paint("github"),
                    fmt.theme().seperator_paint(':'),
                    fmt.theme().arguments_paint(user),
                    fmt.theme().seperator_paint('/'),
                    fmt.theme().arguments_paint(repository),
                )?;
            }
            Gnu => {
                // SAFETY: unwrapping path is safe for this kind.
                let path = unsafe { self.path().unwrap_unchecked() };

                write!(
                    fmt,
                    "{}{}{}",
                    fmt.theme().arguments_paint("gnu"),
                    fmt.theme().seperator_paint(':'),
                    path
                )?;
            }
        }

        Ok(())
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

                    Ok(Source::github(user, repository))
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

                    Ok(Source::gitlab(user, repository))
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
