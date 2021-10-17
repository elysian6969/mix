use crate::Result;
use mix_atom::Requirement;
use mix_config::Config;
use mix_id::{PackageId, RepositoryId};
use mix_manifest::Manifest;
use mix_source::Sources;
use mix_version::Versions;
use path::{Path, PathBuf};
use std::borrow::Borrow;
use std::cmp::Ord;
use std::collections::BTreeSet;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PackageRef {
    /// Ditto.
    pub repository_id: RepositoryId,

    /// Ditto.
    pub package_id: PackageId,

    /// Installed versions.
    pub versions: Versions,

    /// Packages this package depends on.
    pub dependencies: BTreeSet<Requirement>,

    /// Sources which provide this package.
    pub sources: Sources,

    /// Cached manifest path.
    pub manifest_path: PathBuf,

    /// Cached build prefix.
    pub build_prefix: PathBuf,
}

impl PackageRef {
    pub async fn new(
        config: Config,
        repository_id: RepositoryId,
        package_id: PackageId,
    ) -> Result<Self> {
        let build_prefix = config
            .build_prefix()
            .join(repository_id.as_str())
            .join(package_id.as_str());

        let manifest_path = config
            .repos_prefix()
            .join(repository_id.as_str())
            .join(package_id.as_str())
            .join("manifest.yml");

        let manifest_string = manifest_path.read_to_string_async().await?;
        let manifest = Manifest::from_str(manifest_string.as_str())?;
        let mut sources = Sources::new(config.cache_prefix());

        for source in manifest.sources.into_iter() {
            sources.insert(source);
        }

        Ok(Self {
            repository_id,
            package_id,
            versions: Versions::new(),
            sources,
            dependencies: manifest.dependencies,
            manifest_path,
            build_prefix,
        })
    }

    pub fn new_orphaned(
        config: Config,
        repository_id: RepositoryId,
        package_id: PackageId,
    ) -> Self {
        let build_prefix = config
            .build_prefix()
            .join(repository_id.as_str())
            .join(package_id.as_str());

        let manifest_path = config
            .repos_prefix()
            .join(repository_id.as_str())
            .join(package_id.as_str())
            .join("manifest.yml");

        let sources = Sources::new(config.cache_prefix());

        Self {
            repository_id,
            package_id,
            versions: Versions::new(),
            sources,
            dependencies: BTreeSet::new(),
            manifest_path,
            build_prefix,
        }
    }

    pub fn repository_id(&self) -> &RepositoryId {
        &self.repository_id
    }

    pub fn package_id(&self) -> &PackageId {
        &self.package_id
    }

    pub fn dependencies(&self) -> &BTreeSet<Requirement> {
        &self.dependencies
    }

    pub fn get_dependency<Q>(&self, atom: &Q) -> Option<&Requirement>
    where
        Requirement: Borrow<Q>,
        Q: Ord,
    {
        self.dependencies.get(atom)
    }

    pub fn has_dependency<Q>(&self, atom: &Q) -> bool
    where
        Requirement: Borrow<Q>,
        Q: Ord,
    {
        self.get_dependency(atom).is_some()
    }

    pub fn sources(&self) -> &Sources {
        &self.sources
    }

    pub fn sources_mut(&mut self) -> &mut Sources {
        &mut self.sources
    }

    pub fn versions(&self) -> &Versions {
        &self.versions
    }

    pub fn versions_mut(&mut self) -> &mut Versions {
        &mut self.versions
    }

    pub fn installed(&self) -> bool {
        !self.versions.is_empty()
    }

    pub fn manifest_path(&self) -> &Path {
        self.manifest_path.as_path()
    }
    pub fn build_prefix(&self) -> &Path {
        self.build_prefix.as_path()
    }
}

impl Borrow<RepositoryId> for PackageRef {
    fn borrow(&self) -> &RepositoryId {
        self.repository_id()
    }
}

impl Borrow<PackageId> for PackageRef {
    fn borrow(&self) -> &PackageId {
        self.package_id()
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Package(pub(crate) Arc<PackageRef>);

impl Package {
    pub async fn new(
        config: Config,
        repository_id: RepositoryId,
        package_id: PackageId,
    ) -> Result<Self> {
        Ok(Self(Arc::new(
            PackageRef::new(config, repository_id, package_id).await?,
        )))
    }

    pub fn repository_id(&self) -> &RepositoryId {
        self.0.repository_id()
    }

    pub fn package_id(&self) -> &PackageId {
        self.0.package_id()
    }

    pub fn dependencies(&self) -> &BTreeSet<Requirement> {
        self.0.dependencies()
    }

    pub fn get_dependency<Q>(&self, atom: &Q) -> Option<&Requirement>
    where
        Requirement: Borrow<Q>,
        Q: Ord,
    {
        self.0.get_dependency(atom)
    }

    pub fn has_dependency<Q>(&self, atom: &Q) -> bool
    where
        Requirement: Borrow<Q>,
        Q: Ord,
    {
        self.0.has_dependency(atom)
    }

    pub fn sources(&self) -> &Sources {
        self.0.sources()
    }

    pub fn versions(&self) -> &Versions {
        self.0.versions()
    }

    pub fn build_prefix(&self) -> &Path {
        self.0.build_prefix()
    }

    pub fn installed(&self) -> bool {
        self.0.installed()
    }
}

impl From<PackageRef> for Package {
    fn from(package: PackageRef) -> Self {
        Self(Arc::new(package))
    }
}

impl From<Arc<PackageRef>> for Package {
    fn from(package: Arc<PackageRef>) -> Self {
        Self(package)
    }
}

impl Borrow<RepositoryId> for Package {
    fn borrow(&self) -> &RepositoryId {
        self.0.repository_id()
    }
}

impl Borrow<PackageId> for Package {
    fn borrow(&self) -> &PackageId {
        self.0.package_id()
    }
}
