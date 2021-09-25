use futures_util::stream::{BoxStream, StreamExt, TryStreamExt};
use metadata::{GroupId, Package, PackageId};
use path::{Path, PathBuf};
use regex::Regex;
use semver::Version;
use semver::VersionReq;
use std::collections::HashMap;
use std::fmt;
use std::io;
use std::str::FromStr;
use triple::Triple;

pub use crate::set::Set;
pub use crate::shared::Shared;

mod set;
mod shared;

pub struct Packages {
    by_triple: HashMap<Triple, Set>,
    by_group: HashMap<GroupId, Set>,
    by_package: HashMap<PackageId, Set>,
    by_version: HashMap<Version, Set>,
    by_exact: Set,
}

impl Packages {
    pub fn new() -> Self {
        let by_triple = HashMap::new();
        let by_group = HashMap::new();
        let by_package = HashMap::new();
        let by_version = HashMap::new();
        let by_exact = Set::new();

        Self {
            by_triple,
            by_group,
            by_package,
            by_version,
            by_exact,
        }
    }

    pub async fn from_path(prefix: impl AsRef<Path>) -> io::Result<Self> {
        let prefix = prefix.as_ref();
        let mut packages = Self::new();
        let mut dirs = read_dirs(&prefix).await?;

        while let Some(dir) = dirs.try_next().await? {
            if dir
                .file_name()
                .and_then(|dir| dir.to_str())
                .and_then(|dir| Triple::from_str(dir).ok())
                .is_none()
            {
                continue;
            }

            let mut dirs = read_dirs(dir.as_path()).await?;

            while let Some(dir) = dirs.try_next().await? {
                let mut dirs = read_dirs(dir.as_path()).await?;

                while let Some(dir) = dirs.try_next().await? {
                    let mut dirs = read_dirs(dir.as_path()).await?;

                    while let Some(dir) = dirs.try_next().await? {
                        let result = Package::from_path(&prefix, dir.as_path());
                        let package = match result {
                            Ok(package) => package,
                            Err(_) => continue,
                        };

                        packages.insert(package);
                    }
                }
            }
        }

        Ok(packages)
    }

    pub fn insert(&mut self, package: impl Into<Shared>) {
        let package = package.into();

        self.by_triple
            .entry(package.triple())
            .and_modify(|set| set.insert(package.clone()))
            .or_insert_with(|| {
                let mut set = Set::new();

                set.insert(package.clone());
                set
            });

        self.by_group
            .entry(package.group().clone())
            .and_modify(|set| set.insert(package.clone()))
            .or_insert_with(|| {
                let mut set = Set::new();

                set.insert(package.clone());
                set
            });

        self.by_package
            .entry(package.package().clone())
            .and_modify(|set| set.insert(package.clone()))
            .or_insert_with(|| {
                let mut set = Set::new();

                set.insert(package.clone());
                set
            });

        self.by_version
            .entry(package.version().clone())
            .and_modify(|set| set.insert(package.clone()))
            .or_insert_with(|| {
                let mut set = Set::new();

                set.insert(package.clone());
                set
            });

        self.by_exact.insert(package);
    }

    pub fn get_by_triple(&self, triple: &Triple) -> Option<&Set> {
        self.by_triple.get(triple)
    }

    pub fn get_by_group(&self, group: &GroupId) -> Option<&Set> {
        self.by_group.get(group)
    }

    pub fn get_by_package(&self, package: &PackageId) -> Option<&Set> {
        self.by_package.get(package)
    }

    pub fn get_by_version(&self, version: &Version) -> Option<&Set> {
        self.by_version.get(version)
    }

    pub fn get_matches_group<'a>(&'a self, regex: &'a Regex) -> impl Iterator<Item = &'a Shared> {
        self.by_group
            .iter()
            .filter(|(group, _shared)| group.matches(regex))
            .flat_map(|(_group, shared)| shared.iter())
    }

    pub fn get_matches_package<'a>(&'a self, regex: &'a Regex) -> impl Iterator<Item = &'a Shared> {
        self.by_package
            .iter()
            .filter(|(package, _shared)| package.matches(regex))
            .flat_map(|(_package, shared)| shared.iter())
    }

    pub fn get_matches_version<'a>(
        &'a self,
        requirement: &'a VersionReq,
    ) -> impl Iterator<Item = &'a Shared> {
        self.by_version
            .iter()
            .filter(|(version, _shared)| requirement.matches(version))
            .flat_map(|(_version, shared)| shared.iter())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Shared> {
        self.by_exact.iter()
    }

    pub fn remove(&mut self, package: impl Into<Shared>) {
        let package = package.into();

        self.by_triple
            .entry(package.triple())
            .and_modify(|set| set.remove(package.clone()));

        self.by_group
            .entry(package.group().clone())
            .and_modify(|set| set.remove(package.clone()));

        self.by_package
            .entry(package.package().clone())
            .and_modify(|set| set.remove(package.clone()));

        self.by_version
            .entry(package.version().clone())
            .and_modify(|set| set.remove(package.clone()));

        self.by_exact.remove(package);
    }
}

impl fmt::Debug for Packages {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_set().entries(self.by_exact.iter()).finish()
    }
}

async fn read_dirs<'a>(path: impl AsRef<Path>) -> io::Result<BoxStream<'a, io::Result<PathBuf>>> {
    let stream = path
        .as_ref()
        .read_dir()
        .await?
        .try_filter_map(|entry| async move {
            let result = match entry.file_type().await?.is_dir() {
                true => Some(entry.path()),
                false => None,
            };

            Ok(result)
        })
        .boxed();

    Ok(stream)
}
