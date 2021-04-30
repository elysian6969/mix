use crate::args::Fetch;
use crate::package::{Graph, PackageId, UTF8_SYMBOLS};
use crate::PREFIX;
use std::path::Path;

pub async fn fetch(fetch: Fetch, _http: &reqwest::Client) -> anyhow::Result<()> {
    let packages = Path::new(PREFIX).join("repository");

    if !packages.exists() {
        println!();
        println!("==> \x1b[38;5;9mERROR:\x1b[m repository is missing, did you sync?");
        return Ok(());
    }

    let graph = Graph::open(&packages).await?;

    for package_id in fetch.packages.into_iter().map(PackageId::new) {
        print!("{}", graph.display(&package_id, &UTF8_SYMBOLS));
    }

    Ok(())
}
