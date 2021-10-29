use futures_util::stream::{BoxStream, StreamExt, TryStreamExt};
use mix_atom::Atom;
use mix_config::Config;
use mix_id::{PackageId, RepositoryId};
use mix_triple::Triple;
use mix_version::Version;
use path::{Path, PathBuf};
use regex::Regex;
use std::borrow::Borrow;
use std::cmp::Ord;
use std::collections::HashMap;
use std::hash::Hash;
use std::str::FromStr;
use std::{fmt, io};

pub use crate::set::Set;
pub use crate::shared::{Package, PackageRef};

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub mod atoms;
pub mod matches;
pub mod packages;
pub mod set;
mod shared;

pub struct Packages {
    repository_id: HashMap<RepositoryId, Set>,
    package_id: HashMap<PackageId, Set>,
    installed: Set,
    not_installed: Set,
    all: Set,
}

impl Packages {
    /// Create a new packages? lol?
    pub fn new() -> Self {
        let repository_id = HashMap::new();
        let package_id = HashMap::new();
        let installed = Set::new();
        let not_installed = Set::new();
        let all = Set::new();

        Self {
            repository_id,
            package_id,
            installed,
            not_installed,
            all,
        }
    }

    /// Load a tree of packages from the config provided.
    pub async fn from_config(config: &Config) -> io::Result<Self> {
        let mut list: HashMap<(RepositoryId, PackageId), PackageRef> = HashMap::new();
        let _ = config.repos_prefix().create_dir_all_async().await;
        let mut repository_dirs = read_dirs(config.repos_prefix()).await?;

        while let Some(repository_dir) = repository_dirs.try_next().await? {
            let repository_id = match repository_dir
                .file_name()
                .and_then(|id| id.to_str())
                .and_then(|id| RepositoryId::try_from(id).ok())
            {
                Some(id) => id,
                None => continue,
            };

            let mut package_dirs = read_dirs(repository_dir.as_path()).await?;

            while let Some(package_dir) = package_dirs.try_next().await? {
                let package_id = match package_dir
                    .file_name()
                    .and_then(|id| id.to_str())
                    .and_then(|id| PackageId::try_from(id).ok())
                {
                    Some(id) => id,
                    None => continue,
                };

                let future =
                    PackageRef::new(config.clone(), repository_id.clone(), package_id.clone());

                let package = match future.await {
                    Ok(package) => package,
                    Err(_) => continue,
                };

                list.insert((repository_id.clone(), package_id.clone()), package);
            }
        }

        let mut triple_dirs = read_dirs(config.prefix()).await?;

        while let Some(triple_dir) = triple_dirs.try_next().await? {
            if triple_dir
                .file_name()
                .and_then(|triple| triple.to_str())
                .and_then(|triple| Triple::from_str(triple).ok())
                .is_none()
            {
                continue;
            }

            let mut repository_dirs = read_dirs(triple_dir.as_path()).await?;

            while let Some(repository_dir) = repository_dirs.try_next().await? {
                let repository_id = match repository_dir
                    .file_name()
                    .and_then(|id| id.to_str())
                    .and_then(|id| RepositoryId::try_from(id).ok())
                {
                    Some(id) => id,
                    None => continue,
                };

                let mut package_dirs = read_dirs(repository_dir.as_path()).await?;

                while let Some(package_dir) = package_dirs.try_next().await? {
                    let package_id = match package_dir
                        .file_name()
                        .and_then(|id| id.to_str())
                        .and_then(|id| PackageId::try_from(id).ok())
                    {
                        Some(id) => id,
                        None => continue,
                    };

                    let mut version_dirs = read_dirs(package_dir.as_path()).await?;

                    while let Some(version_dir) = version_dirs.try_next().await? {
                        let version = match version_dir
                            .file_name()
                            .and_then(|id| id.to_str())
                            .and_then(|id| Version::parse(id).ok())
                        {
                            Some(id) => id,
                            None => continue,
                        };

                        if let Some(package) =
                            list.get_mut(&(repository_id.clone(), package_id.clone()))
                        {
                            package.versions_mut().insert(version, version_dir);
                        } else {
                            let mut package = PackageRef::new_orphaned(
                                config.clone(),
                                repository_id.clone(),
                                package_id.clone(),
                            );

                            package.versions_mut().insert(version, version_dir);
                            list.insert((repository_id.clone(), package_id.clone()), package);
                        }
                    }
                }
            }
        }

        let mut packages = Self::new();

        for (_, package) in list {
            packages.insert(package);
        }

        Ok(packages)
    }

    /// Add a package to this list.
    pub fn insert(&mut self, package: impl Into<Package>) {
        let package = package.into();

        self.repository_id
            .entry(package.repository_id().clone())
            .and_modify(|set| set.insert(package.clone()))
            .or_insert_with(|| {
                let mut set = Set::new();

                set.insert(package.clone());
                set
            });

        self.package_id
            .entry(package.package_id().clone())
            .and_modify(|set| set.insert(package.clone()))
            .or_insert_with(|| {
                let mut set = Set::new();

                set.insert(package.clone());
                set
            });

        if package.installed() {
            self.installed.insert(package.clone());
        } else {
            self.not_installed.insert(package.clone());
        }

        self.all.insert(package);
    }

    /// Return a set of packages within a repository.
    pub fn repository<Q>(&self, id: &Q) -> Option<&Set>
    where
        RepositoryId: Borrow<Q>,
        Q: Ord + Hash,
    {
        self.repository_id.get(id)
    }

    /// Return a set of packages within a repository.
    pub fn repository_mut<Q>(&mut self, id: &Q) -> Option<&mut Set>
    where
        RepositoryId: Borrow<Q>,
        Q: Ord + Hash,
    {
        self.repository_id.get_mut(id)
    }

    /// Return a set of packages with this `id`.
    pub fn packages<Q>(&self, id: &Q) -> Option<&Set>
    where
        PackageId: Borrow<Q>,
        Q: Ord + Hash,
    {
        self.package_id.get(id)
    }

    /// Returns a set of packages with this `id`, iterator variant.
    pub fn packages_iter<'a, Q>(&'a self, id: &Q) -> packages::Packages<'a>
    where
        PackageId: Borrow<Q>,
        Q: Ord + Hash,
    {
        let iter = self
            .packages(id)
            .into_iter()
            .flat_map(packages::map_set_to_iter as packages::MapSetToIter<'a>);

        packages::Packages { iter }
    }

    /// Get a package by `repository_id` and `package_id`.
    pub fn get(&self, repository_id: &RepositoryId, package_id: &PackageId) -> Option<&Package> {
        self.repository(repository_id)
            .and_then(|set| set.get(package_id))
    }

    /// Get a package by `repository_id` and `package_id`.
    pub fn get_mut(
        &mut self,
        repository_id: &RepositoryId,
        package_id: &PackageId,
    ) -> Option<&mut Package> {
        self.repository_mut(repository_id)
            .and_then(|set| set.get_mut(package_id))
    }

    /// Get matching atoms.
    pub fn atoms<'a>(&'a self, atom: &'a Atom) -> atoms::Atoms<'a> {
        if let Some(repository_id) = &atom.repository_id {
            let iter = self.get(&repository_id, &atom.package_id).into_iter();

            atoms::Atoms::Exact(iter)
        } else {
            let iter = self.packages_iter(&atom.package_id);

            atoms::Atoms::Set(iter)
        }
    }

    /// Get matches.
    pub fn matches<'a>(&'a self, requirement: &'a mix_atom::Requirement) -> matches::Matches<'a> {
        let iter = if let Some(repository_id) = &requirement.repository_id {
            let iter = self
                .get(&repository_id, &requirement.package_id)
                .into_iter();

            atoms::Atoms::Exact(iter)
        } else {
            let iter = self.packages_iter(&requirement.package_id);

            atoms::Atoms::Set(iter)
        };

        matches::Matches { iter }
    }

    /// Get matching repositories.
    pub fn matches_repository<'a>(&'a self, regex: &'a Regex) -> impl Iterator<Item = &'a Package> {
        self.repository_id
            .iter()
            .filter(|(repository_id, _shared)| repository_id.matches(regex))
            .flat_map(|(_repository_id, shared)| shared.iter())
    }

    /// Get matching packages.
    pub fn matches_package<'a>(&'a self, regex: &'a Regex) -> impl Iterator<Item = &'a Package> {
        self.package_id
            .iter()
            .filter(|(package_id, _shared)| package_id.matches(regex))
            .flat_map(|(_package_id, shared)| shared.iter())
    }

    /// An iterator over all packages.
    pub fn iter(&self) -> set::Iter<'_> {
        self.all.iter()
    }

    pub fn remove_package<Q>(&mut self, id: &Q)
    where
        Package: Borrow<Q>,
        Q: Ord + Hash,
    {
        if let Some(package) = self.all.get(id) {
            if let Some(set) = self.repository_id.get_mut(package.repository_id()) {
                set.remove(id);
            }

            if let Some(set) = self.package_id.get_mut(package.package_id()) {
                set.remove(id);
            }

            if package.installed() {
                self.installed.remove(id);
            } else {
                self.not_installed.remove(id);
            }
        }

        self.all.remove(id);
    }

    pub fn len(&self) -> usize {
        self.all.len()
    }

    pub fn is_empty(&self) -> bool {
        self.all.is_empty()
    }

    /// List of packages that are installed.
    pub fn installed(&self) -> &Set {
        &self.installed
    }

    /// List of packages that are not installed.
    pub fn not_installed(&self) -> &Set {
        &self.installed
    }
}

impl fmt::Debug for Packages {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_set().entries(self.all.iter()).finish()
    }
}

pub async fn read_dirs<'a>(
    path: impl AsRef<Path>,
) -> io::Result<BoxStream<'a, io::Result<PathBuf>>> {
    let stream = path
        .as_ref()
        .read_dir_async()
        .await?
        .try_filter_map(|entry| async move { Ok(entry.is_dir_async().await.then(|| entry.path())) })
        .boxed();

    Ok(stream)
}
