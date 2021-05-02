use crate::shell::Shell;
use reqwest::Client;

pub struct Config {
    client: Client,
    shell: Shell,
}

impl Config {
    pub fn new() -> crate::Result<Self> {
        let client = Client::builder().user_agent(crate::user_agent()).build()?;

        Ok(Self {
            client,
            shell: Shell::default(),
        })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn shell(&self) -> &Shell {
        &self.shell
    }
}
