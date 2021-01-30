use crate::source::Source;
use crate::util;
use futures::stream::{StreamExt, TryStreamExt};
use serde::Deserialize;
use std::cell::{Ref, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::rc::{Rc, Weak};
use tokio::fs;
use tokio::fs::DirEntry;

#[derive(Debug, Deserialize)]
pub struct Metadata {
    depends: Option<BTreeSet<String>>,
    source: BTreeSet<Source>,
}

#[derive(Debug)]
pub struct Package {
    name: String,
    metadata: Metadata,
    depends: RefCell<BTreeMap<String, Weak<Package>>>,
}

impl Package {
    pub fn depends(&self) -> Ref<'_, BTreeMap<String, Weak<Package>>> {
        self.depends.borrow()
    }
}

impl From<(String, Metadata)> for Package {
    fn from((name, metadata): (String, Metadata)) -> Self {
        Self {
            name,
            metadata,
            depends: RefCell::new(BTreeMap::new()),
        }
    }
}

#[derive(Debug)]
pub struct Repository {
    packages: BTreeMap<String, Rc<Package>>,
}

impl Repository {
    pub async fn open(path: &Path) -> anyhow::Result<Self> {
        let path = path.join("packages");
        let packages: BTreeMap<String, Rc<Package>> = util::read_dir(path)
            .await?
            .then(|entry| async move {
                let entry = entry?;
                let name = entry
                    .file_name()
                    .into_string()
                    .map_err(|_| anyhow::anyhow!("invalid utf-8"))?;

                let config = entry.path().join("package.yml");
                let slice = &fs::read(config).await?;
                let metadata: Metadata = serde_yaml::from_slice(&slice)?;
                let package = Package::from((name.clone(), metadata));

                Ok::<_, anyhow::Error>((name, Rc::new(package)))
            })
            .try_collect()
            .await?;

        for (name, package) in &packages {
            if let Some(package) = packages.get(name) {
                if let Some(depends) = &package.metadata.depends {
                    for depend in depends {
                        if let Some(depend_package) = packages.get(depend.as_str()) {
                            let mut depends = package.depends.borrow_mut();

                            depends.insert(depend.clone(), Rc::downgrade(&depend_package));
                        }
                    }
                }
            }
        }

        Ok(Self { packages })
    }

    pub fn get(&self, name: &str) -> Option<&Rc<Package>> {
        self.packages.get(name)
    }
}
