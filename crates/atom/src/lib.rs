use id::{PackageId, RepositoryId};
use semver::{Version, VersionReq};
use std::cmp::Ordering;
use std::str::FromStr;
use std::{cmp, fmt};

#[cfg(feature = "serde")]
mod serde;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Atom {
    repository_id: Option<RepositoryId>,
    package_id: PackageId,
    version: Version,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AtomReq {
    repository_id: Option<RepositoryId>,
    package_id: PackageId,
    version_hint: VersionReq,
}

impl FromStr for Atom {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
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

        let repository_id = if repository_id.is_empty() {
            None
        } else {
            Some(repository_id.into())
        };

        // SAFETY: `slash + 1` and `colon` is guarenteed to be a valid position within `input`.
        //         `slash + 1` is guarenteed to be <= `colon`.
        let package_id = unsafe {
            input.get_unchecked(slash.saturating_add((slash > 0) as usize).min(colon)..colon)
        };

        if package_id.is_empty() {
            return Err("Expected package ID.");
        }

        // SAFETY: `colon + 1` is guarenteed to be a valid position within `input`.
        //         `colon + 1` is guarenteed to be <= `input.len()`.
        let version = unsafe { input.get_unchecked(colon.saturating_add(1).min(input.len())..) };
        let version = Version::parse(version);
        let version = match version {
            Ok(version) => version,
            Err(_) => return Err("Invalid version"),
        };

        Ok(Self {
            repository_id,
            package_id: package_id.into(),
            version,
        })
    }
}

impl FromStr for AtomReq {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
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
            Some(repository_id.into())
        };

        // SAFETY: `slash + 1` and `colon` is guarenteed to be a valid position within `input`.
        //         `slash + 1` is guarenteed to be <= `colon`.
        let package_id = unsafe {
            input.get_unchecked(slash.saturating_add((slash > 0) as usize).min(colon)..colon)
        };

        if package_id.is_empty() {
            return Err("Expected package ID.");
        }

        // SAFETY: `colon + 1` is guarenteed to be a valid position within `input`.
        //         `colon + 1` is guarenteed to be <= `input.len()`.
        let version_hint =
            unsafe { input.get_unchecked(colon.saturating_add(1).min(input.len())..) };
        let version_hint = VersionReq::parse(version_hint).unwrap_or(VersionReq::STAR);

        Ok(Self {
            repository_id,
            package_id: package_id.into(),
            version_hint,
        })
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if let Some(repository_id) = &self.repository_id {
            fmt.write_str(repository_id.as_str())?;
            fmt.write_str("/")?;
        }

        fmt.write_str(self.package_id.as_str())?;
        fmt.write_str(":")?;
        fmt.write_fmt(format_args!("{}", &self.version))?;

        Ok(())
    }
}

impl fmt::Display for AtomReq {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if let Some(repository_id) = &self.repository_id {
            fmt.write_str(repository_id.as_str())?;
            fmt.write_str("/")?;
        }

        fmt.write_str(self.package_id.as_str())?;

        if self.version_hint != VersionReq::STAR {
            fmt.write_str(":")?;
            fmt.write_fmt(format_args!("{}", &self.version_hint))?;
        }

        Ok(())
    }
}

impl cmp::Ord for AtomReq {
    fn cmp(&self, other: &Self) -> Ordering {
        self.repository_id
            .cmp(&other.repository_id)
            .then(self.package_id.cmp(&other.package_id))
    }
}

impl cmp::PartialOrd for AtomReq {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
