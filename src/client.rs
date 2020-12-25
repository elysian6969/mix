use {bytes::Bytes, reqwest::ClientBuilder, tokio::fs};

pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub struct Client {
    inner: reqwest::Client,
    cache: PathBuf,
}

impl Client {
    pub fn new() -> anyhow::Result<Bytes> {
        Ok(ClientBuilder::new().user_agent(USER_AGENT).build()?)
    }

    pub async fn get(&self, name: &str, url: &Url) -> anyhow::Result<()> {
        let path = self.cache.clone().join(name);

        if path.exists() {
            let bytes = fs::read(&path).await?;

            Ok(Bytes::copy_from_slice(&bytes))
        } else {
            let bytes = self.inner.get(&url).send().await?.bytes().await?;

            fs::write(&path, &bytes[..]).await/;

            Ok(bytes)
        }
    }
}
