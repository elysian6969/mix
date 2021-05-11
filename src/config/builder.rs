use super::{Client, Config};
use crate::shell::Shell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use url::Url;

pub struct Builder {
    pub(super) prefix: PathBuf,
    pub(super) repositories: BTreeMap<PathBuf, Url>,
    pub(super) error: crate::Result<()>,
    pub(super) user_agent: String,
}

impl Builder {
    pub fn repository(mut self, path: impl Into<PathBuf>, url: impl AsRef<str>) -> Self {
        match Url::parse(url.as_ref()) {
            Ok(url) => {
                self.repositories.insert(path.into(), url);
            }
            Err(err) => self.error = self.error.and(Err(Box::new(err))),
        }

        self
    }

    pub fn repositories(
        mut self,
        repositories: impl IntoIterator<Item = (impl Into<PathBuf>, impl AsRef<str>)>,
    ) -> Self {
        for (path, url) in repositories {
            self = self.repository(path, url);
        }

        self
    }

    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    pub fn build(self) -> crate::Result<Config> {
        self.error?;

        let client = reqwest::Client::builder()
            .user_agent(self.user_agent)
            .build()?;

        Ok(Config {
            client: Client { client },
            build: self.prefix.join("build"),
            cache: self.prefix.join("cache"),
            prefix: self.prefix,
            repositories: self.repositories,
            shell: Shell::default(),
        })
    }
}
