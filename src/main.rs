use {
    clap::Clap,
    std::{fs::File, io::Read, path::PathBuf},
};

pub mod shell;
pub mod spec;
pub mod triple;
pub mod util;
pub mod watson;

#[derive(Clap, Debug)]
pub struct Args {
    #[clap(parse(from_os_str))]
    spec: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmdline = Args::parse();

    let mut spec = File::open(&cmdline.spec)?;
    let metnya = spec.metadata()?;
    let mut buffy = vec![0; metnya.len() as usize];

    spec.read(&mut buffy)?;

    let candy = watson::Watson::new(&"/milk");
    let spec: spec::Spec = serde_yaml::from_slice(&buffy)?;

    spec.execute(&candy, &triple::X86_64_UNKNOWN_LINUX_GNU)
        .await?;

    Ok(())
}
