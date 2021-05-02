use crate::shell::Shell;
use reqwest::Client;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use url::Url;

pub struct Config {
    client: Client,
    prefix: PathBuf,
    repositories: BTreeMap<PathBuf, Url>,
    shell: Shell,
}

impl Config {
    pub fn builder(prefix: impl Into<PathBuf>) -> ClientBuilder {
        ClientBuilder {
            prefix: prefix.into(),
            repositories: BTreeMap::new(),
            error: Ok(()),
            user_agent: crate::user_agent().into(),
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn prefix(&self) -> &Path {
        self.prefix.as_path()
    }

    pub fn repositories(&self) -> &BTreeMap<PathBuf, Url> {
        &self.repositories
    }

    pub fn shell(&self) -> &Shell {
        &self.shell
    }
}

pub struct ClientBuilder {
    prefix: PathBuf,
    repositories: BTreeMap<PathBuf, Url>,
    error: crate::Result<()>,
    user_agent: String,
}

impl ClientBuilder {
    pub fn repository(mut self, path: impl Into<PathBuf>, url: impl AsRef<str>) -> Self {
        match Url::parse(url.as_ref()) {
            Ok(url) => {
                self.repositories.insert(path.into(), url);
            }
            Err(err) => self.error = self.error.and(Err(Box::new(err))),
        }

        self
    }

    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    pub fn build(self) -> crate::Result<Config> {
        self.error?;

        let client = Client::builder().user_agent(self.user_agent).build()?;

        Ok(Config {
            client,
            prefix: self.prefix,
            repositories: self.repositories,
            shell: Shell::default(),
        })
    }
}
