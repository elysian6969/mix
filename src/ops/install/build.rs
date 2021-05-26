use crate::config::Config;
use crate::external;
use crate::external::tar;
use crate::package::{Entry, Node};
use crate::shell::Text;
use crossterm::style::Colorize;
use semver::Version;
use std::cell::{self, RefCell};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

crate struct Ref {
    config: Config,
    build_dir: PathBuf,
    install_dir: PathBuf,
    source_dir: RefCell<PathBuf>,
    tarball: PathBuf,
    version: Version,
    group: String,
    package: String,
}

pub struct Build(Arc<Ref>);

impl Build {
    pub fn new(config: &Config, entry: &Entry, version: &Version, tarball: &Path) -> Self {
        let (build_dir, install_dir) = config.build_dirs(entry.node(), version);

        Self(Arc::new(Ref {
            config: config.clone(),
            source_dir: RefCell::new(build_dir.clone()),
            build_dir,
            install_dir,
            group: entry.node().group_id.as_str().to_string(),
            package: entry.node().package_id.as_str().to_string(),
            version: version.clone(),
            tarball: tarball.to_path_buf(),
        }))
    }

    pub fn config(&self) -> &Config {
        &self.0.config
    }

    pub fn build_dir(&self) -> &Path {
        self.0.build_dir.as_path()
    }

    pub fn source_dir(&self) -> cell::Ref<'_, PathBuf> {
        self.0.source_dir.borrow()
    }

    pub fn install_dir(&self) -> &Path {
        self.0.install_dir.as_path()
    }

    pub fn jobs(&self) -> usize {
        16
    }

    pub fn version(&self) -> &Version {
        &self.0.version
    }

    pub fn tarball(&self) -> &Path {
        self.0.tarball.as_path()
    }

    pub fn group(&self) -> &str {
        self.0.group.as_str()
    }

    pub fn package(&self) -> &str {
        self.0.package.as_str()
    }

    pub async fn build(&self) -> crate::Result<()> {
        if self.install_dir().exists() {
            return Ok(());
        }

        let buffer = unsafe {
            let result = ufmt::uformat!(
                "{}/{} v{}\n",
                self.group().blue().to_string(),
                self.package().green().to_string(),
                &self.version().to_string(),
            );

            result.unwrap_unchecked()
        };

        Text::new(buffer).render(self.config().shell()).await?;

        let _ = fs::remove_dir_all(self.build_dir()).await;
        let entries = tar::extract(self.config(), &self.tarball(), self.build_dir()).await?;

        if let Some(root) = entries.get(0usize) {
            let root = self.build_dir().join(&root);

            if root.join("CMakeLists.txt").exists() {
                *self.0.source_dir.borrow_mut() = root;

                external::cmake().execute(self).await?;
            } else if root.join("meson.build").exists() {
                use external::meson::Subcommand;

                *self.0.source_dir.borrow_mut() = root;

                let args = [
                    "-Dtests=false",
                    "--buildtype=release",
                    "--wrap-mode=nodownload",
                ];

                external::meson().execute(self).await?;

                external::meson()
                    .args(&args)
                    .subcommand(Subcommand::Configure)
                    .execute(self)
                    .await?;

                *self.0.source_dir.borrow_mut() = self.build_dir().to_path_buf();

                external::meson()
                    .subcommand(Subcommand::Compile)
                    .execute(self)
                    .await?;
            } else if root.join("configure").exists() {
                *self.0.source_dir.borrow_mut() = root;

                external::autotools().execute(self).await?;
            } else {
                println!("UNKNOWN BUILD");
            }
        }

        // implement tracking to reduce i/o
        // {build}/{group}/{package}/{version}
        /*let _ = fs::remove_dir_all(self.build_dir()).await;

        for ancestor in self.build_dir().ancestors().take(2usize) {
            let _ = fs::remove_dir(&ancestor).await;
        }*/

        Ok(())
    }
}

impl Deref for Build {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.0.config
    }
}
