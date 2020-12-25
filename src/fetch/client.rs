use {
    ahash::AHasher,
    bytes::Bytes,
    reqwest::ClientBuilder,
    tokio::fs,
    std::{
        hash::Hasher,
        path::{Path, PathBuf},
    },
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

        Ok(Client {
            inner,
            cache,
        })
    }

    pub async fn get(&self, name: impl AsRef<str>, url: impl AsRef<str>) -> anyhow::Result<Bytes> {
        let name = name.as_ref();
        let url = url.as_ref();
        let path = self.cache.clone().join(name);

        dbg!(&path);

        let mut hasher = AHasher::new_with_keys(420, 1337);

        hasher.write(&url);

        let hash = hasher.finish();

        dbg!(&hash);

        fs::create_dir_all(&path).await?;

        if path.exists() {
            let bytes = fs::read(&path).await?;

            Ok(Bytes::copy_from_slice(&bytes))
        } else {
            return Ok(Bytes::new());

            let bytes = self.inner.get(url).send().await?.bytes().await?;

            fs::write(&path, &bytes[..]).await?;

            Ok(bytes)
        }
    }
}
