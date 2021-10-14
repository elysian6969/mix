use futures_util::future;
use mix_id::PackageId;
use path::{Path, PathBuf};
use std::collections::HashMap;
use std::iter;

const CONFIGS: [&str; 26] = [
    "aclocal.m4",
    "autogen",
    "autogen.sh",
    "bootstrap",
    "bootstrap.sh",
    "Cargo.toml",
    "config",
    "config.sh",
    "Config",
    "Config.sh",
    "configure",
    "configure.ac",
    "configure.in",
    "configure.sh",
    "Configure",
    "Configure.sh",
    "CMakeLists.txt",
    "makefile",
    "Makefile",
    "Makefile.am",
    "Makefile.in",
    "meson.build",
    "gnumakefile",
    "GNUmakefile",
    "GNUMakefile",
    "x.py",
];

pub async fn detect(
    package_id: &PackageId,
    current_dir: impl AsRef<Path>,
) -> HashMap<String, PathBuf> {
    let package = package_id.as_str();
    let package_sh = format!("{}.sh", package_id.as_str());
    let current_dir = current_dir.as_ref();
    let package_path = current_dir.join(&package);
    let package_sh_path = current_dir.join(&package_sh);

    let configs: HashMap<_, _> = CONFIGS
        .iter()
        .map(|config| (config.to_string(), current_dir.join(config)))
        .chain([
            (package.to_string(), package_path),
            (package_sh, package_sh_path),
        ])
        .collect();

    let futures = configs.iter().map(|(_, config)| config.exists_async());
    let results = future::join_all(futures).await;

    iter::zip(configs, results)
        .filter_map(|(pair, exists)| exists.then(|| pair))
        .collect()
}
