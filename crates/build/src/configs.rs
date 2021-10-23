use futures_util::future;
use mix_id::PackageId;
use path::{Path, PathBuf};
use std::collections::HashMap;
use std::iter;

const CONFIGS: [&str; 26] = [
    /* 00 */ "aclocal.m4",
    /* 01 */ "autogen",
    /* 02 */ "autogen.sh",
    /* 03 */ "bootstrap",
    /* 04 */ "bootstrap.sh",
    /* 05 */ "Cargo.toml",
    /* 06 */ "config",
    /* 07 */ "config.sh",
    /* 08 */ "Config",
    /* 09 */ "Config.sh",
    /* 10 */ "configure",
    /* 11 */ "configure.ac",
    /* 12 */ "configure.in",
    /* 13 */ "configure.sh",
    /* 14 */ "Configure",
    /* 15 */ "Configure.sh",
    /* 16 */ "CMakeLists.txt",
    /* 17 */ "makefile",
    /* 18 */ "Makefile",
    /* 19 */ "Makefile.am",
    /* 20 */ "Makefile.in",
    /* 21 */ "meson.build",
    /* 22 */ "gnumakefile",
    /* 23 */ "GNUmakefile",
    /* 24 */ "GNUMakefile",
    /* 25 */ "x.py",
];

// Use to determine the build system to compile this package.
pub struct System {
    pub config: HashMap<Box<str>, Box<Path>>,
}

impl System {
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

    pub fn has_autotools(&self) -> bool {
        self.config.contains_key(CONFIGS[00])
            || self.config.contains_key(CONFIGS[01])
            || self.config.contains_key(CONFIGS[02])
            || self.config.contains_key(CONFIGS[03])
            || self.config.contains_key(CONFIGS[04])
            || self.config.contains_key(CONFIGS[10])
            || self.config.contains_key(CONFIGS[11])
            || self.config.contains_key(CONFIGS[12])
            || self.config.contains_key(CONFIGS[13])
            || self.config.contains_key(CONFIGS[19])
            || self.config.contains_key(CONFIGS[20])
    }

    pub fn has_bootstrap(&self) -> bool {
        self.config.contains_key(CONFIGS[03]) || self.config.contains_key(CONFIGS[04])
    }

    pub fn has_cargo(&self) -> bool {
        self.config.contains_key(CONFIGS[05])
    }

    pub fn has_cmake(&self) -> bool {
        self.config.contains_key(CONFIGS[16])
    }

    pub fn has_makefile(&self) -> bool {
        self.config.contains_key(CONFIGS[17])
            || self.config.contains_key(CONFIGS[18])
            || self.config.contains_key(CONFIGS[22])
            || self.config.contains_key(CONFIGS[23])
            || self.config.contains_key(CONFIGS[24])
    }

    pub fn has_meson(&self) -> bool {
        self.config.contains_key(CONFIGS[21])
    }

    pub fn has_rust_bootstrap(&self) -> bool {
        self.config.contains_key(CONFIGS[25])
    }

    pub fn get_autotools(&self) -> Option<(Option<&Path>, Option<&Path>)> {
        if self.has_autotools() {
            let bootstrap = self
                .config
                .get(CONFIGS[03])
                .or_else(|| self.config.get(CONFIGS[04]))
                .map(|path| &**path);

            let configure = self.config.get(CONFIGS[10]).map(|path| &**path);

            Some((bootstrap, configure))
        } else {
            None
        }
    }

    pub fn get_bootstrap(&self) -> Option<&Path> {
        self.config
            .get(CONFIGS[03])
            .or_else(|| self.config.get(CONFIGS[04]))
            .map(|path| &**path)
    }

    pub fn get_cargo(&self) -> Option<&Path> {
        self.config.get(CONFIGS[05]).map(|path| &**path)
    }

    pub fn get_cmake(&self) -> Option<&Path> {
        self.config.get(CONFIGS[16]).map(|path| &**path)
    }

    pub fn get_makefile(&self) -> Option<&Path> {
        self.config
            .get(CONFIGS[17])
            .or_else(|| self.config.get(CONFIGS[18]))
            .or_else(|| self.config.get(CONFIGS[22]))
            .or_else(|| self.config.get(CONFIGS[23]))
            .or_else(|| self.config.get(CONFIGS[24]))
            .map(|path| &**path)
    }

    pub fn get_meson(&self) -> Option<&Path> {
        self.config.get(CONFIGS[21]).map(|path| &**path)
    }

    pub fn get_rust_bootstrap(&self) -> Option<&Path> {
        self.config.get(CONFIGS[25]).map(|path| &**path)
    }
}
