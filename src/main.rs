use {
    self::{config::Config, triple::Triple},
    clap::Clap,
    std::{fs::File, io::Read, path::PathBuf},
};

pub mod config;
pub mod delete_on_drop;
//pub mod shell;
//pub mod spec;
pub mod triple;
//pub mod util;

#[derive(Clap, Debug)]
pub struct Args {
    #[clap(parse(from_os_str))]
    path: PathBuf,
}

pub mod build {
    use {
        super::{config::Config, triple::Triple},
        semver::Version,
        serde::Deserialize,
    };

    #[derive(Debug, Deserialize)]
    pub struct RawScript {
        pub name: String,
        pub version: Version,
        pub source: Option<Vec<String>>,
        pub configure: Option<Vec<String>>,
        pub make: Option<Vec<String>>,
    }

    #[derive(Debug)]
    pub struct Script {
        pub name: String,
        pub version: Version,
        pub source: Option<Vec<String>>,
        pub configure: Option<Vec<String>>,
        pub make: Option<Vec<String>>,
    }

    impl RawScript {
        pub fn preprocess(self, config: &Config, triple: &Triple) -> anyhow::Result<Script> {
            println!("{:#?}", &self);

            Ok(Script {
                name: self.name,
                version: self.version,
                source: self.source,
                configure: self.configure,
                make: self.make,
            })
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = Config::with_prefix("/tiramisu");
    let target = Triple::x86_64().linux().gnu();
    let raw_script: build::RawScript = File::open(args.path).map(serde_yaml::from_reader)??;
    let script = raw_script.preprocess(&config, &target)?;

    println!("{:#?}", &script);

    Ok(())
}
