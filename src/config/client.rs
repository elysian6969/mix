use super::Config;
use crate::partial::Partial;
use crate::shell::{ProgressBar, Text};
use byte_unit::{AdjustedByte, Byte};
use crossterm::cursor::{Hide, MoveToColumn, Show};
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType::CurrentLine};
use futures::stream::StreamExt;
use std::fmt;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::time::Duration;
use tokio::{fs, time};

pub struct Client {
    pub(super) client: reqwest::Client,
}

impl Client {
    pub async fn download(
        &self,
        config: &Config,
        path: impl AsRef<Path>,
        url: impl AsRef<str>,
    ) -> crate::Result<()> {
        let path = Partial::new(path.as_ref());

        // TODO; implement continue
        if !path.whole().exists() {
            let mut interval = time::interval(Duration::from_millis(500));
            let mut downloaded = 0;

            render(config, path.partial(), display_bytes(downloaded)).await?;

            let mut destination = File::create(path.partial()).await?;
            let response = self.client.get(url.as_ref()).send().await?;
            let length = response.content_length();
            let mut stream = response.bytes_stream();

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Some(length) = length {
                            let progress = (downloaded as u64 / length) as f32;

                            ProgressBar::new(0.0f32..=100.0f32, progress)
                                .render(config.shell())
                                .await?;
                        }

                        render(config, path.partial(), display_bytes(downloaded)).await?;
                    }
                    bytes = stream.next() => if let Some(bytes) = bytes {
                        let bytes = bytes?;
                        let bytes = &bytes[..];

                        downloaded += bytes.len();
                        destination.write_all(&bytes).await?;
                    } else {
                        break;
                    }
                }
            }

            destination.flush().await?;
            render(config, path.partial(), "downloaded!").await?;
            Text::new(Show.to_string()).render(config.shell()).await?;
            fs::rename(path.partial(), path.whole()).await?;
        }

        Ok(())
    }
}

fn display_bytes(bytes: usize) -> AdjustedByte {
    Byte::from(bytes as u64).get_appropriate_unit(false)
}

async fn render(config: &Config, path: &Path, bytes: impl fmt::Display) -> crate::Result<()> {
    let buffer = format!(
        "{hide}{clear}{move_to}{space}{path}{seperator}{status}",
        hide = Hide,
        clear = Clear(CurrentLine),
        move_to = MoveToColumn(0),
        space = "    ",
        path = path.display(),
        seperator = ": ",
        status = Print(bytes),
    );

    Text::new(buffer).render(config.shell()).await?;

    Ok(())
}
