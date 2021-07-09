use crate::config::Config;
use crate::external;
use crate::external::tar;
use crate::package::{Entry, Node};
use crate::shell::Text;
use crossterm::style::Stylize;
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

        Text::new(buffer)
            .render(self.config().shell())
            .await
            .unwrap();

        let _ = fs::remove_dir_all(self.build_dir()).await;
        let entries = tar::extract(self.config(), &self.tarball(), self.build_dir())
            .await
            .unwrap();

        if let Some(root) = entries.get(0usize) {
            let root = self.build_dir().join(&root);

            if root.join("CMakeLists.txt").exists() {
                *self.0.source_dir.borrow_mut() = root;

                cmake_wrap::configure(self.source_dir().deref(), self.build_dir())
                    .prefix_dir(self.install_dir())
                    .spawn()?
                    .wait()
                    .await?;

                cmake_wrap::build(self.build_dir())
                    .jobs(8)
                    .spawn()?
                    .wait()
                    .await?;

                cmake_wrap::install(self.build_dir())
                    .spawn()?
                    .wait()
                    .await?;
            } else if root.join("meson.build").exists() {
                *self.0.source_dir.borrow_mut() = root;

                let build_dir = self.build_dir().join("build");
                let _ = fs::create_dir(&build_dir).await;

                meson_wrap::configure(self.source_dir().deref(), &build_dir)
                    .tests(false)
                    .build_kind("release")
                    .prefix_dir(self.install_dir())
                    .wrap_kind("nodownload")
                    .spawn()?
                    .wait()
                    .await?;

                meson_wrap::build(&build_dir).spawn()?.wait().await?;

                meson_wrap::install(&build_dir).spawn()?.wait().await?;
            } else if root.join("bootstrap").exists()
                || root.join("configure.ac").exists()
                || root.join("configure").exists()
                || root.join("Makefile.ac").exists()
            {
                *self.0.source_dir.borrow_mut() = root;

                let mut aclocal = autotools_wrap::aclocal(self.source_dir().deref());

                let m4 = self.source_dir().deref().join("m4");

                if m4.exists() {
                    aclocal.include(m4);
                }

                aclocal.spawn()?.wait().await?;

                autotools_wrap::autoconf(self.source_dir().deref())
                    .spawn()?
                    .wait()
                    .await?;

                autotools_wrap::autoheader(self.source_dir().deref())
                    .spawn()?
                    .wait()
                    .await?;

                autotools_wrap::automake(self.source_dir().deref())
                    .spawn()?
                    .wait()
                    .await?;

                autotools_wrap::bootstrap(self.source_dir().deref())
                    .spawn()?
                    .wait()
                    .await?;

                autotools_wrap::configure(self.source_dir().deref())
                    .spawn()?
                    .wait()
                    .await?;
            } else {
                println!("UNKNOWN BUILD {entries:#?}");
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
