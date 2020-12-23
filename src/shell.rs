
use crossterm::style::{Colorize, Styler};
use std::fmt;

/// Action to print to the terminal
#[derive(Clone, Copy, Debug)]
pub enum Action {
    Building,
    Installing,
    Preparing,
    Running,
    Updating,
}

impl Action {
    pub fn as_str(&self) -> &str {
        match self {
            Action::Building => "building",
            Action::Installing => "installing",
            Action::Preparing => "preparing",
            Action::Running => "running",
            Action::Updating => "updating",
        }
    }

    pub fn to_display(&self) -> impl fmt::Display {
        format!("{: >12}", self.as_str().to_owned().green().bold())
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
