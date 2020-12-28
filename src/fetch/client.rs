use {
    byte_unit::Byte,
    bytes::Bytes,
    crossterm::{cursor, style, terminal, QueueableCommand},
    futures::stream::StreamExt,
    reqwest::ClientBuilder,
    std::{
        borrow::Cow,
        io::Write,
        path::{Path, PathBuf},
        time::Instant,
    },
    tokio::{
        fs::{self, File},
        io::AsyncWriteExt,
    },
};

pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub struct Client {
    inner: reqwest::Client,
    cache: PathBuf,
}

impl Client {
    pub fn with_cache(path: impl AsRef<Path>) -> anyhow::Result<Client> {
        let inner = ClientBuilder::new().user_agent(USER_AGENT).build()?;
        let cache = path.as_ref().to_path_buf();

        Ok(Client { inner, cache })
    }

    pub fn cache(&self) -> Cow<Path> {
        Cow::from(&self.cache)
    }

    pub async fn get(
        &self,
        package: impl AsRef<str>,
        name: impl AsRef<str>,
        url: impl AsRef<str>,
    ) -> anyhow::Result<Bytes> {
        let package = package.as_ref();
        let name = name.as_ref();
        let url = url.as_ref();
        let package = self.cache.clone().join(package);
        let path = package.clone().join(name);

        fs::create_dir_all(&package).await?;

        if path.exists() {
            let bytes = fs::read(&path).await?;

            Ok(Bytes::copy_from_slice(&bytes))
        } else {
            let bytes = self.inner.get(url).send().await?.bytes().await?;

            fs::write(&path, &bytes[..]).await?;

            Ok(bytes)
        }
    }

    pub async fn get_partial(
        &self,
        package: impl AsRef<str>,
        name: impl AsRef<str>,
        url: impl AsRef<str>,
    ) -> anyhow::Result<()> {
        let package = package.as_ref();
        let name = name.as_ref();
        let url = url.as_ref();
        let package = self.cache.clone().join(package);
        let partial_name = format!("{}.partial", name);
        let path = package.clone().join(&name);
        let partial_path = package.clone().join(&partial_name);

        fs::create_dir_all(&package).await?;

        if path.exists() {
            Ok(())
        } else {
            {
                let mut amount = 0u128;
                let mut instant = Instant::now();
                let mut stdout = std::io::stdout();

                let adj_byte = Byte::from_bytes(amount).get_appropriate_unit(false);

                stdout
                    .queue(cursor::Hide)?
                    .queue(terminal::Clear(terminal::ClearType::CurrentLine))?
                    .queue(cursor::MoveToColumn(0))?
                    .queue(style::Print("    "))?
                    .queue(style::Print(&partial_name))?
                    .queue(style::Print(": "))?
                    .queue(style::Print(adj_byte))?
                    .flush()?;

                let mut file = File::create(&partial_path).await?;
                let mut stream = self.inner.get(url).send().await?.bytes_stream();

                while let Some(bytes) = stream.next().await {
                    let bytes = bytes?;

                    amount += bytes.len() as u128;

                    if instant.elapsed().as_millis() > 200 {
                        instant = Instant::now();

                        let adj_byte = Byte::from_bytes(amount).get_appropriate_unit(false);

                        stdout
                            .queue(terminal::Clear(terminal::ClearType::CurrentLine))?
                            .queue(cursor::MoveToColumn(0))?
                            .queue(style::Print("    "))?
                            .queue(style::Print(&partial_name))?
                            .queue(style::Print(": "))?
                            .queue(style::Print(adj_byte))?
                            .flush()?;
                    }

                    let bytes = &bytes[..];

                    file.write_all(&bytes).await?;
                }

                file.flush().await?;

                let adj_byte = Byte::from_bytes(amount).get_appropriate_unit(false);

                stdout
                    .queue(terminal::Clear(terminal::ClearType::CurrentLine))?
                    .queue(cursor::MoveToColumn(0))?
                    .queue(style::Print("    "))?
                    .queue(style::Print(&partial_name))?
                    .queue(style::Print(": "))?
                    .queue(style::Print(adj_byte))?
                    .queue(style::Print(" downloaded!"))?
                    .queue(cursor::Show)?
                    .flush()?;
            }

            fs::rename(&partial_path, &path).await?;

            Ok(())
        }
    }
}
