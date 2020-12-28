use {
    self::{config::Config, fetch::Client, triple::Triple},
    clap::Clap,
    semver::Version,
    std::{fs::File, iter, path::PathBuf},
};

pub mod build;
pub mod config;
pub mod delete_on_drop;
pub mod fetch;
pub mod triple;

#[derive(Clap, Debug)]
pub enum Args {
    Build(Build),
}

#[derive(Clap, Debug)]
pub struct Build {
    #[clap(parse(from_os_str))]
    packages: Vec<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = Config::new("/tiramisu", iter::once(Triple::default()));
    let client = Client::with_cache("/tiramisu/cache")?;
    let current_version = Version::parse("5.0.0")?;

    match &args {
        Args::Build(build) => {
            for package in &build.packages {
                let script = File::open(&package).map(serde_yaml::from_reader)??;

                for target in config.targets() {
                    build::build(
                        &package,
                        &script,
                        &config,
                        &target,
                        &client,
                        &current_version,
                    )
                    .await?;
                }
            }
        }
    }

    Ok(())
}
