#![feature(format_args_capture)]

use crate::options::Options;
use packages::Packages;
use path::Path;
use regex::Regex;
use std::time::Instant;
use tokio::runtime::Builder;

mod options;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
type Result<T, E = Error> = std::result::Result<T, E>;

fn default_prefix() -> &'static Path {
    Path::new("/milk")
}

async fn run() -> Result<()> {
    group::Package::from_path("/milk", "/milk/repo/core/glibc/metadata.yml").await?;

    let _options = Options::parse();
    let now = Instant::now();
    let packages = Packages::from_path(default_prefix()).await?;
    let elapsed = now.elapsed();

    println!(">>> loading packages took {elapsed:?}");

    for package in packages.iter() {
        println!("    {package:?}");
    }

    let now = Instant::now();
    let regex = Regex::new("f").unwrap();
    let matches = packages.get_matches_package(&regex);
    let elapsed = now.elapsed();

    println!(">>> resolve took {elapsed:?}");

    for package in matches {
        println!("    {package:?}");
    }

    Ok(())
}

fn main() -> Result<()> {
    Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(run())?;

    Ok(())
}
