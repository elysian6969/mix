#![feature(const_trait_impl)]
#![feature(map_first_last)]

pub use crate::error::Error;
pub use crate::versions::Versions;
use mix_shell::{async_trait, write, AsyncDisplay, Shell};
use semver::{BuildMetadata, Prerelease};
use std::str::FromStr;
use std::{fmt, mem};

mod error;
pub mod versions;

#[cfg(feature = "serde")]
mod serde;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Version {
    pub(crate) version: semver::Version,
}

impl Version {
    pub fn major(&self) -> u64 {
        self.version.major
    }

    pub fn minor(&self) -> u64 {
        self.version.minor
    }

    pub fn patch(&self) -> u64 {
        self.version.patch
    }

    pub fn pre(&self) -> &Prerelease {
        &self.version.pre
    }

    pub fn build(&self) -> &BuildMetadata {
        &self.version.build
    }

    pub fn matches(&self, requirement: &Requirement) -> bool {
        requirement.matches(self)
    }

    pub fn parse(text: &str) -> Result<Self, Error> {
        semver::Version::parse(text)
            .map(|version| Self { version })
            .map_err(|error| unsafe { mem::transmute(error) })
    }
}

impl fmt::Debug for Version {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.version, fmt)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.version, fmt)
    }
}

#[async_trait(?Send)]
impl AsyncDisplay<Shell> for Version {
    async fn fmt(&self, fmt: &Shell) -> mix_shell::Result<()> {
        write!(
            fmt,
            "{}{}{}{}{}",
            fmt.theme().arguments_paint(self.major()),
            fmt.theme().seperator_paint('.'),
            fmt.theme().arguments_paint(self.minor()),
            fmt.theme().seperator_paint('.'),
            fmt.theme().arguments_paint(self.patch())
        )?;

        if !self.pre().is_empty() {
            write!(
                fmt,
                "{}{}",
                fmt.theme().seperator_paint('-'),
                fmt.theme().arguments_paint(self.pre())
            )?;
        }

        if !self.build().is_empty() {
            write!(
                fmt,
                "{}{}",
                fmt.theme().seperator_paint('-'),
                fmt.theme().arguments_paint(self.build())
            )?;
        }

        Ok(())
    }
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text)
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Requirement {
    requirement: semver::VersionReq,
}

impl Requirement {
    pub const fn star() -> Self {
        let requirement = semver::VersionReq::STAR;

        Self { requirement }
    }

    pub fn parse(text: &str) -> Result<Self, Error> {
        semver::VersionReq::parse(text)
            .map(|requirement| Self { requirement })
            .map_err(|error| unsafe { mem::transmute(error) })
    }

    pub fn matches(&self, version: &Version) -> bool {
        self.requirement.matches(&version.version)
    }

    pub fn is_star(&self) -> bool {
        self.requirement == Self::star().requirement
    }
}

impl fmt::Debug for Requirement {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.requirement, fmt)
    }
}

impl fmt::Display for Requirement {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.requirement, fmt)
    }
}

impl FromStr for Requirement {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text)
    }
}

impl const Default for Requirement {
    fn default() -> Self {
        Self::star()
    }
}
