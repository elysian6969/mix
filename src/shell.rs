use crossterm::style::{Colorize, Styler};
use std::fmt;

/// current action
#[derive(Clone, Copy, Debug)]
pub enum Action {
    Fetch,
    Config,
    Build,
    Install,
}

impl Action {
    pub fn as_str(&self) -> &str {
        use Action::*;

        match self {
            Fetch => "fetch",
            Config => "config",
            Build => "build",
            Install => "install",
        }
    }

    pub fn to_display(&self) -> impl fmt::Display {
        format!("-> {}", self.as_str().to_owned().dark_green().bold())
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_display())
    }
}

/// Action to print to the terminal
#[derive(Clone, Copy, Debug)]
pub enum Status {
    Error,
    Warning,
}

impl Status {
    pub fn as_str(&self) -> &str {
        match self {
            Status::Error => "error",
            Status::Warning => "warning",
        }
    }

    pub fn to_display(&self) -> impl fmt::Display {
        let text = match self {
            Status::Error => "error".red(),
            Status::Warning => "warning".yellow(),
        };

        format!("{}:", text)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_display())
    }
}
