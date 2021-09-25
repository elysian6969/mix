use crate::options::Options;
use tokio::runtime::Builder;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

mod options;

async fn async_main() -> Result<()> {
    let options = Options::parse();

    match options {
        Options::Build(build) => milk_build::build(build.into_config()).await?,
    }

    Ok(())
}

fn main() -> Result<()> {
    Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async_main())
}

pub mod shell {
    use std::fmt::Display;
    use yansi::{Color, Style};

    pub struct Styles {
        decoration: &'static str,
        decoration_style: Style,
        action_style: Style,
        arguments_style: Style,
        command_style: Style,
        output_style: Style,
        output_err_style: Style,
    }

    impl Default for Styles {
        fn default() -> Self {
            Self {
                decoration: " >",
                decoration_style: Style::new(Color::White).dimmed(),
                action_style: Style::default(),
                arguments_style: Style::new(Color::Magenta),
                command_style: Style::new(Color::Green),
                output_style: Style::default(),
                output_err_style: Style::new(Color::Red),
            }
        }
    }

    pub fn header(styles: &Styles, action: impl Display, arguments: impl Display) {
        println!(
            "{decoration} {action: <13} {arguments}",
            decoration = styles.decoration_style.paint(&styles.decoration),
            action = styles.action_style.paint(&action),
            arguments = styles.arguments_style.paint(&arguments),
        );
    }

    pub fn command_out(styles: &Styles, command: impl Display, output: impl Display) {
        println!(
            "{decoration} {command} {output}",
            decoration = styles.decoration_style.paint(&styles.decoration),
            command = styles.command_style.paint(&command),
            output = styles.output_style.paint(&output),
        );
    }

    pub fn command_err(styles: &Styles, command: impl Display, output: impl Display) {
        println!(
            "{decoration} {command} {output}",
            decoration = styles.decoration_style.paint(&styles.decoration),
            command = styles.command_style.paint(&command),
            output = styles.output_err_style.paint(&output),
        );
    }
}
