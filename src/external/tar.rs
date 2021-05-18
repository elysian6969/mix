use crate::shell::Text;
use crate::Config;
use std::path::Path;
use std::process::Stdio;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

pub async fn extract(
    config: &Config,
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
) -> crate::Result<()> {
    let buffer = ufmt::uformat!(" -> extracting {:?}\n", src.as_ref()).expect("infallible");

    Text::new(buffer).render(config.shell()).await?;

    fs::create_dir_all(dst.as_ref()).await?;

    let mut child = Command::new("bsdtar")
        .arg("xfv")
        .arg(src.as_ref())
        .current_dir(dst)
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    let stderr = child
        .stderr
        .take()
        .expect("child did not have a handle to stderr");

    let stdout = child
        .stdout
        .take()
        .expect("child did not have a handle to stdout");

    let mut stderr = BufReader::new(stderr).lines();
    let mut stdout = BufReader::new(stdout).lines();

    tokio::spawn(async move {
        // handle errors and status
        let _ = child.wait().await;
    });

    while let Some(line) = stderr.next_line().await? {
        //println!("line: {}", line);
    }

    while let Some(line) = stdout.next_line().await? {
        //println!("line: {}", line);
    }

    Ok(())
}
