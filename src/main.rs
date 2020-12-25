use {
    self::{config::Config, triple::Triple},
    clap::Clap,
    std::{fs::File, path::PathBuf},
};

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

pub mod build {
    use {
        super::{config::Config, triple::Triple},
        semver::Version,
        serde::Deserialize,
        std::path::PathBuf,
        url::Url,
    };

    #[derive(Debug, Deserialize)]
    pub struct Script {
        pub version: Version,
        pub source: Vec<Url>,
        pub configure: Option<Vec<String>>,
        pub make: Option<Vec<String>>,
    }

    pub async fn build(
        path: &PathBuf,
        script: &Script,
        config: &Config,
        triple: &Triple,
    ) -> anyhow::Result<()> {
        println!("path: {:?}", &path);
        println!("script: {:?}", &script);
        println!("config: {:?}", &config);
        println!("triple: {:?}", &triple);

        for source in &script.source {
            match source.scheme() {
                "github" => {
                    let mut segments = source.path().split('/');

                    let user = segments
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("invalid github user"))?;

                    let repo = segments
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("invalid github repo"))?;

                    let available = crate::fetch::github::fetch_github_tags(&user, &repo).await?;

                    println!("available: {:?}", &available);
                }
                _ => Err(anyhow::anyhow!("invalid source"))?,
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = Config::with_prefix("/tiramisu");
    let target = Triple::x86_64().linux().gnu();

    match &args {
        Args::Build(build) => {
            for package in &build.packages {
                let script = File::open(&package).map(serde_yaml::from_reader)??;

                build::build(&package, &script, &config, &target).await?;
            }
        }
    }

    Ok(())
}
