use metadata::{GroupId, PackageId};
use path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use std::path::Components;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Metadata {
    source: Vec<String>,
    depend: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Package {
    group: GroupId,
    package: PackageId,
    path: PathBuf,
    source: Vec<String>,
    depend: Vec<String>,
}

impl Package {
    pub async fn from_path(prefix: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<Self> {
        let prefix = prefix.as_ref();
        let path = path.as_ref();
        let stripped = path.strip_prefix(prefix)?;
        let mut components = stripped.components();

        unsafe fn extend_ref<'a, 'b, T: ?Sized>(val: &'a T) -> &'b T {
            &*(val as *const T)
        }

        fn next<'a>(components: *mut Components<'a>) -> Option<&'a Path> {
            let path = unsafe { extend_ref((&mut *components).next()?.as_ref()) };

            Some(path)
        }

        fn to_str(path: &Path) -> Result<&str, &'static str> {
            path.to_str().ok_or_else(|| "Expected UTF-8 in string.")
        }

        let group = next(&mut components).ok_or("Expected group.")?;
        let package = next(&mut components).ok_or("Expected package.")?;

        let group = to_str(group)?;
        let package = to_str(package)?;

        let group = GroupId::new(group);
        let package = PackageId::new(package);
        let bytes = path.read().await?;

        let Metadata { source, depend, .. } = serde_yaml::from_slice(&bytes)?;

        Ok(Self {
            group,
            package,
            path: path.to_path_buf(),
            source,
            depend,
        })
    }
}

use futures_util::stream::{BoxStream, StreamExt, TryStreamExt};
use metadata::{GroupId, PackageId};
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

//mod set;
//mod shared;

pub struct Packages {
    by_group: HashMap<GroupId, Set>,
    by_package: HashMap<PackageId, Set>,
    by_source: HashMap<String, Set>,
    by_depend: HashMap<String, Set>,
    by_exact: Set,
}

impl Packages {
    pub fn new() -> Self {
        let by_group = HashMap::new();
        let by_package = HashMap::new();
        let by_source = HashMap::new();
        let by_depend = HashMap::new();
        let by_exact = Set::new();

        Self {
            by_group,
            by_package,
            by_source,
            by_depend,
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

        self.by_source
            .entry(package.version().clone())
            .and_modify(|set| set.extend(package.source()))
            .or_insert_with(|| {
                let mut set = Set::new();

                set.extend(package.source());
                set
            });

        self.by_depend
            .entry(package.version().clone())
            .and_modify(|set| set.extend(package.source()))
            .or_insert_with(|| {
                let mut set = Set::new();

                set.extend(package.source());
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
