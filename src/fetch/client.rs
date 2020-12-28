use {
    bytes::Bytes,
    reqwest::ClientBuilder,
    std::{
        hash::Hasher,
        path::{Path, PathBuf},
    },
    tokio::fs,
    url::Url,
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
}
