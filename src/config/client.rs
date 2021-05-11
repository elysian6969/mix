use crossterm::cursor::{Hide, MoveToColumn};
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType::CurrentLine};
use std::path::Path;

pub struct Client {
    pub(super) client: reqwest::Client,
}

impl Client {
    pub async fn download(&self, path: impl AsRef<Path>) -> crate::Result<()> {
        let path = Partial::new(path);

        // TODO; implement continue
        if path.whole().exists() {
            Ok(())
        } else {
            let mut interval = time::interval(Duration::from_millis(500));
            let mut downloaded = 0;

            draw(config, path.partial(), display_bytes(downloaded)).await?;

            let mut destination = File::create(path.partial()).await?;
            let response = self.client.get(&url).send().await?;
            let length = response.content_length();
            let mut stream = response.bytes_stream();

            if let Some(length) = length {
                let progress = (amount / length) as f32;

                ProgressBar::new(0.0f32..=100.0f32, progress)
                    .draw(&shell)
                    .await?;

                loop {
                    tokio::select! {
                        _ = &mut interval => {
                            draw(config, partial.partial(), display_bytes(downloaded)).await?;
                        }
                        bytes = stream.next() => {
                            let bytes = bytes?;
                            let bytes = &bytes[..];

                            downloaded += bytes.len();
                            destination.write_all(&bytes).await?;
                        }
                    }
                }
            } else {
                loop {
                    tokio::select! {
                        _ = &mut interval => {
                            draw(config, partial.partial(), display_bytes(downloaded)).await?;
                        }
                        bytes = stream.next() => {
                            let bytes = bytes?;
                            let bytes = &bytes[..];

                            downloaded += bytes.len();
                            destination.write_all(&bytes).await?;
                        }
                    }
                }
            }

            destination.flush().await?;
            draw(config, path.partial(), "downloaded!")?;
            stdout.queue(Show)?.flush()?;
        };

        fs::rename(partial.partial(), partial.whole()).await?;

        Ok(())
    }
}

fn display_bytes(bytes: usize) -> AdjustedByte {
    Byte::from(bytes as u64).get_appropriate_unit(false)
}

async fn draw(config: &Config, path: &Path, bytes: impl Display) -> anyhow::Result<()> {
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
