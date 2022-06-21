use futures_util::future;
use mix_id::PackageId;
use path::{Path, PathBuf};
use std::collections::HashMap;
use std::iter;

const ACLOCAL: &str = "aclocal.m4";
const AUTOGEN: &str = "autogen";
const AUTOGEN_SH: &str = "autogen.sh";
const BOOTSTRAP: &str = "bootstrap";
const BOOTSTRAP_SH: &str = "bootstrap.sh";
const BUILD_AUTOGEN_SH: &str = "build/autogen.sh";
const CARGO: &str = "Cargo.toml";
const CONFIG: &str = "config";
const CONFIG_SH: &str = "config.sh";
const CONFIG_UPPER: &str = "Config";
const CONFIG_UPPER_SH: &str = "Config.sh";
const CONFIGURE: &str = "configure";
const CONFIGURE_AC: &str = "configure.ac";
const CONFIGURE_IN: &str = "configure.in";
const CONFIGURE_SH: &str = "configure.sh";
const CONFIGURE_UPPER: &str = "Configure";
const CONFIGURE_UPPER_SH: &str = "Configure";
const CMAKE: &str = "CMakeLists.txt";
const MAKEFILE: &str = "makefile";
const MAKEFILE_UPPER: &str = "Makefile";
const MAKEFILE_AM: &str = "Makefile.am";
const MAKEFILE_IN: &str = "Makefile.in";
const MESON: &str = "meson.build";
const GNUMAKEFILE: &str = "gnumakefile";
const GNUMAKEFILE_UPPER: &str = "GNUmakefile";
const GNUMAKEFILE_UPPER2: &str = "GNUMakefile";
const RUST_BOOTSTRAP: &str = "x.py";

const CONFIGS: [&str; 27] = [
    ACLOCAL,
    AUTOGEN,
    AUTOGEN_SH,
    BOOTSTRAP,
    BOOTSTRAP_SH,
    BUILD_AUTOGEN_SH,
    CARGO,
    CONFIG,
    CONFIG_SH,
    CONFIG_UPPER,
    CONFIG_UPPER_SH,
    CONFIGURE,
    CONFIGURE_AC,
    CONFIGURE_IN,
    CONFIGURE_SH,
    CONFIGURE_UPPER,
    CONFIGURE_UPPER_SH,
    CMAKE,
    MAKEFILE,
    MAKEFILE_UPPER,
    MAKEFILE_AM,
    MAKEFILE_IN,
    MESON,
    GNUMAKEFILE,
    GNUMAKEFILE_UPPER,
    GNUMAKEFILE_UPPER2,
    RUST_BOOTSTRAP,
];

// Use to determine the build system to compile this package.
pub struct System {
    pub config: HashMap<Box<str>, Box<Path>>,
}

impl System {
    #[inline]
    pub async fn new(id: &PackageId, dir: impl AsRef<Path>) -> Self {
        let id = id.as_str();
        let dir = dir.as_ref();
        let id_path = [dir, Path::new(id)].into_iter().collect::<PathBuf>();
        let sh_path = id_path.with_extension("sh");
        let sh = unsafe { sh_path.file_name().unwrap_unchecked().as_str().to_string() };

        let configs: HashMap<_, _> = CONFIGS
            .iter()
            .map(|config| (config.to_string(), dir.join(config)))
            .chain([(id.to_string(), id_path), (sh, sh_path)])
            .collect();

        let futures = configs.iter().map(|(_, config)| config.exists_async());
        let results = future::join_all(futures).await;
        let config = iter::zip(configs, results)
            .filter_map(|((name, config), exists)| {
                exists.then(|| (name.into_boxed_str(), config.into_boxed_path()))
            })
            .collect();

        Self { config }
    }

    #[inline]
    pub fn needs_aclocal(&self) -> bool {
        self.config.contains_key(ACLOCAL) && self.config.contains_key(CONFIGURE_AC)
    }

    #[inline]
    pub fn has_autotools(&self) -> bool {
        self.needs_aclocal()
            || self.config.contains_key(AUTOGEN)
            || self.config.contains_key(AUTOGEN_SH)
            || self.config.contains_key(CONFIGURE)
            || self.config.contains_key(CONFIGURE_AC)
            || self.config.contains_key(CONFIGURE_IN)
            || self.config.contains_key(CONFIGURE_SH)
            || self.config.contains_key(MAKEFILE_AM)
            || self.config.contains_key(MAKEFILE_IN)
            || self.has_bootstrap()
    }

    #[inline]
    pub fn has_bootstrap(&self) -> bool {
        self.config.contains_key(BOOTSTRAP) || self.config.contains_key(BOOTSTRAP_SH)
    }

    #[inline]
    pub fn has_cargo(&self) -> bool {
        self.config.contains_key(CARGO)
    }

    #[inline]
    pub fn has_cmake(&self) -> bool {
        self.config.contains_key(CMAKE)
    }

    #[inline]
    pub fn has_configure_ac(&self) -> bool {
        self.config.contains_key(CONFIGURE_AC)
    }

    #[inline]
    pub fn has_makefile(&self) -> bool {
        self.config.contains_key(MAKEFILE)
            || self.config.contains_key(MAKEFILE_UPPER)
            || self.config.contains_key(GNUMAKEFILE)
            || self.config.contains_key(GNUMAKEFILE_UPPER)
            || self.config.contains_key(GNUMAKEFILE_UPPER2)
    }

    #[inline]
    pub fn has_meson(&self) -> bool {
        self.config.contains_key(MESON)
    }

    #[inline]
    pub fn has_rust_bootstrap(&self) -> bool {
        self.config.contains_key(RUST_BOOTSTRAP)
    }

    #[inline]
    pub fn get_autogen(&self) -> Option<&Path> {
        self.config
            .get(AUTOGEN)
            .or_else(|| self.config.get(AUTOGEN_SH))
            .or_else(|| self.config.get(BUILD_AUTOGEN_SH))
            .map(as_ref)
    }

    #[inline]
    pub fn get_autotools(&self) -> Option<(Option<&Path>, Option<&Path>)> {
        if self.has_autotools() {
            let bootstrap = self.get_bootstrap();
            let configure = self.get_autotools_configure();

            Some((bootstrap, configure))
        } else {
            None
        }
    }

    #[inline]
    pub fn get_autotools_configure(&self) -> Option<&Path> {
        self.config.get(CONFIGURE).map(as_ref)
    }

    #[inline]
    pub fn get_bootstrap(&self) -> Option<&Path> {
        self.config
            .get(BOOTSTRAP)
            .or_else(|| self.config.get(BOOTSTRAP_SH))
            .map(as_ref)
    }

    #[inline]
    pub fn get_cargo(&self) -> Option<&Path> {
        self.config.get(CARGO).map(as_ref)
    }

    #[inline]
    pub fn get_configure(&self) -> Option<&Path> {
        self.config.get(CONFIGURE).map(as_ref)
    }

    #[inline]
    pub fn get_cmake(&self) -> Option<&Path> {
        self.config.get(CMAKE).map(as_ref)
    }

    #[inline]
    pub fn get_makefile(&self) -> Option<&Path> {
        self.config
            .get(MAKEFILE)
            .or_else(|| self.config.get(MAKEFILE_UPPER))
            .or_else(|| self.config.get(GNUMAKEFILE))
            .or_else(|| self.config.get(GNUMAKEFILE_UPPER))
            .or_else(|| self.config.get(GNUMAKEFILE_UPPER2))
            .map(as_ref)
    }

    #[inline]
    pub fn get_meson(&self) -> Option<&Path> {
        self.config.get(MESON).map(as_ref)
    }

    #[inline]
    pub fn get_rust_bootstrap(&self) -> Option<&Path> {
        self.config.get(RUST_BOOTSTRAP).map(as_ref)
    }
}

#[inline]
fn as_ref(path: &Box<Path>) -> &Path {
    &**path
}
