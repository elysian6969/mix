use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use url::Url;

#[derive(Deserialize)]
pub struct Metadata {
    pub repositories: BTreeMap<PathBuf, Url>,
}

impl Metadata {
    pub async fn open(path: impl AsRef<Path>) -> crate::Result<Self> {
        let slice = &fs::read(path).await?;
        let metadata = serde_yaml::from_slice(&slice)?;

        Ok(metadata)
    }
}

impl ufmt::uDebug for Metadata {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        struct Repositories<'repos>(&'repos BTreeMap<PathBuf, Url>);

        impl<'repos> ufmt::uDebug for Repositories<'repos> {
            fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
            where
                W: ufmt::uWrite + ?Sized,
            {
                f.debug_map()?
                    .entries(self.0.iter().map(|(path, url)| (path, url.as_str())))?
                    .finish()
            }
        }

        f.debug_struct("Metadata")?
            .field("repositories", &Repositories(&self.repositories))?
            .finish()
    }
}
