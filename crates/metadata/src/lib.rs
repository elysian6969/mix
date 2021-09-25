use path::{Path, PathBuf};
use regex::Regex;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::path::Components;
use triple::Triple;

pub use crate::package_id::PackageId;
pub use crate::repository_id::RepositoryId;

mod package_id;
mod repository_id;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InstalledMetadata {
    triple: Triple,
    version: Version,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Metadata {
    source: Vec<String>,
    depend: Vec<String>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Dependency {
    repository_id: RepositoryId,
    package_id: PackageId,
    version_hint: Option<VersionReq>,
}

impl Dependency {
    pub fn new(repository_id: impl Into<RepositoryId>, package_id: impl Into<PackageId>) -> Self {
        Self::new_with_hint(repository_id, package_id, None)
    }

    pub fn new_with_hint(
        repository_id: impl Into<RepositoryId>,
        package_id: impl Into<PackageId>,
        version_hint: impl Into<Option<VersionReq>>,
    ) -> Self {
        let repository_id = repository_id.into();
        let package_id = packags_id.into();
        let version_hint = version_hint.into();

        Self {
            repository_id,
            package_id,
            version_hint,
        }
    }
}

#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PackageId {
    /// The repository this package belongs to.
    repository_id: RepositoryId,

    /// This package's identifier.
    package_id: PackageId,

    /// Metadata for this package.
    ///
    /// `{prefix}/repo/{repository_id}/{package_name}/metadata.yml`
    metadata_path: Option<Box<Path>>,

    /// Installed paths for this package. Dependant on triple and version.
    ///
    /// `{prefix}/{triple}/{repository_id}/{package_name}/{version}`
    installed_paths: BTreeMap<InstalledMetadata, Box<Path>>,

    /// List of sources to build this package.
    sources: BTreeSet<Box<str>>,

    /// List of dependencies for this package.
    dependencies: BTreeMap<Dependency>,
}

pub impl PackageId {
    pub fn new(repository_id: RepositoryId, package_id: PackageId) -> Self {
        Self {
            repository_id,
            package_id,
            metadata_path: None,
            installed_paths: BTreeMap::new(),
            sources: Vec::new(),
            dependencies: BTteeSet::new(),
        }
    }

    pub async fn set_metadata(
        &mut self,
        prefix: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> Result<()> {
        let prefix = prefix.as_ref();
        let path = path.as_ref();
        let stripped = path.strip_prefix(prefix)?;
        let mut components = StringComponents(stripped.components()).flatten();

        let group = components.next().ok_or("Expected group.")?;
        let package = components.next().ok_or("Expected package.")?;

        let group = RepositoryId::new(group);
        let package = PackageId::new(package);
        let metadata = path.read_async().await?;
        let Metadata { source, depend, .. } = serde_yaml::from_slice(&metadata)?;

        self.metadata_path = Some(path.to_path_buf());

        for source in source {
            self.sources.insert(source);
        }

        for dependency in depend {
            self.dependencies.insert(Dependency {
                repository_id: RepositoryId::new("<default>"),
                package_id: dependenc,
            })
        }
    }

    pub fn parse_installed_path(prefix: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<Self> {
        let prefix = prefix.as_ref();
        let path = path.as_ref();
        let stripped = path.strip_prefix(prefix)?;
        let mut components = StringComponents(stripped.components()).flatten();

        let triple = components.next().ok_or("Expected triple.")?;
        let group = components.next().ok_or("Expected group.")?;
        let package = components.next().ok_or("Expected package.")?;
        let version = components.next().ok_or("Expected version.")?;

        let triple: Triple = triple.parse()?;
        let group = RepositoryId::new(group);
        let package = PackageId::new(package);
        let version = Version::parse(version)?;

        Ok(Self {
            group,
            package,
            path: path.to_path_buf(),
            triple,
            version,
        })
    }

    pub fn group(&self) -> &RepositoryId {
        &self.group
    }

    pub fn package(&self) -> &PackageId {
        &self.package
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn triple(&self) -> Triple {
        self.triple
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn matches_group(&self, regex: &Regex) -> bool {
        self.group.matches(regex)
    }

    pub fn matches_package(&self, regex: &Regex) -> bool {
        self.package.matches(regex)
    }

    pub fn matches_version(&self, requirement: &VersionReq) -> bool {
        requirement.matches(self.version())
    }

    pub async fn exists(&self) -> bool {
        self.path().exists_async().await
    }
}

struct StringComponents<'a>(Components<'a>);

impl<'a> Iterator for Wrapper<'a> {
    type Item = Result<&'a str>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next().map(|component| {
            let path: &Path = component.as_ref();

            path.to_str().ok_or("Expected UTF-8 in string.")
        })
    }
}

unsafe fn extend_ref<'a, 'b, T: ?Sized>(val: &'a T) -> &'b T {
    &*(val as *const T)
}
