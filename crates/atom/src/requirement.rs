pub use crate::error::Error;
use mix_id::{PackageId, RepositoryId};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::convert::TryInto;
use std::str::FromStr;
use std::{cmp, fmt};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Requirement {
    pub repository_id: Option<RepositoryId>,
    pub package_id: PackageId,
    pub requirement: mix_version::Requirement,
}

impl Requirement {
    pub fn parse(input: &str) -> Result<Self, Error> {
        // If there is no slash, set to start of input.
        let slash = input.find('/').unwrap_or(0);

        // SAFETY: `slash` is guarenteed to be a valid position within `input`.
        let suffix = unsafe { input.get_unchecked(slash..) };

        // Find a colon within the suffix, after the slash, if present.
        // Normalize it's offset due to clipping.
        // If there is no coloh, set to end of input.
        let colon = suffix
            .find(':')
            .map(|index| index.saturating_add(slash))
            .unwrap_or(input.len());

        // SAFETY: `slash` is guarenteed to be a valid position within `input`.
        let repository_id = unsafe { input.get_unchecked(..slash) };

        let repository_id = if repository_id.is_empty() || repository_id == "*" {
            None
        } else {
            Some(repository_id.try_into()?)
        };

        // SAFETY: `slash + 1` and `colon` is guarenteed to be a valid position within `input`.
        //         `slash + 1` is guarenteed to be <= `colon`.
        let package_id = unsafe {
            input.get_unchecked(slash.saturating_add((slash > 0) as usize).min(colon)..colon)
        };

        if package_id.is_empty() {
            return Err(Error::ExpectedPackageId);
        }

        // SAFETY: `colon + 1` is guarenteed to be a valid position within `input`.
        //         `colon + 1` is guarenteed to be <= `input.len()`.
        let requirement =
            unsafe { input.get_unchecked(colon.saturating_add(1).min(input.len())..) };
        let requirement = mix_version::Requirement::parse(requirement).unwrap_or_default();

        Ok(Self {
            repository_id,
            package_id: package_id.try_into()?,
            requirement,
        })
    }
}

impl FromStr for Requirement {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::parse(input)
    }
}

impl fmt::Display for Requirement {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if let Some(repository_id) = &self.repository_id {
            fmt.write_str(repository_id.as_str())?;
            fmt.write_str("/")?;
        }

        fmt.write_str(self.package_id.as_str())?;

        if !self.requirement.is_star() {
            fmt.write_str(":")?;
            fmt.write_fmt(format_args!("{}", &self.requirement))?;
        }

        Ok(())
    }
}

impl cmp::Ord for Requirement {
    fn cmp(&self, other: &Self) -> Ordering {
        self.repository_id
            .cmp(&other.repository_id)
            .then(self.package_id.cmp(&other.package_id))
    }
}

impl cmp::PartialOrd for Requirement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Borrow<PackageId> for Requirement {
    fn borrow(&self) -> &PackageId {
        &self.package_id
    }
}
