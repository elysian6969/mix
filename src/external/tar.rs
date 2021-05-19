use super::process::{Command, Stdio};
use crate::shell::Text;
use crate::Config;
use crossterm::style::Colorize;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};

pub async fn extract(
    config: &Config,
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
) -> crate::Result<Vec<PathBuf>> {
    let buffer = ufmt::uformat!(
        " -> {} {:?}\n",
        "extract".yellow().to_string(),
        src.as_ref()
    )
    .expect("infallible");

    Text::new(buffer).render(config.shell()).await?;

    let _ = fs::create_dir_all(dst.as_ref()).await;

    let mut command = Command::new("bsdtar");

    command
        .arg("xfv")
        .arg(src.as_ref())
        .current_dir(dst)
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .stdout(Stdio::piped());

    let mut child = command.spawn()?;

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

    let mut entries = vec![];

    while let Some(line) = stderr.next_line().await? {
        let entry = line.trim_start_matches('x').trim().into();

        entries.push(entry);
    }

    while let Some(line) = stdout.next_line().await? {
        //println!("line: {}", line);
    }

    Ok(entries)
}
