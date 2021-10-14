use yansi::{Color, Paint, Style};

#[derive(Debug)]
pub struct Theme {
    action_style: Style,
    arguments_style: Style,
    command_style: Style,
    header_prefix: &'static str,
    header_prefix_style: Style,
    error_style: Style,
    output_style: Style,
    seperator_style: Style,
    url_style: Style,
    warning_style: Style,
}

impl Theme {
    pub fn new() -> Self {
        Self {
            action_style: Style::default(),
            arguments_style: Style::new(Color::Magenta),
            command_style: Style::new(Color::Green),
            error_style: Style::new(Color::Red),
            header_prefix: " > ",
            header_prefix_style: Style::new(Color::White).dimmed(),
            output_style: Style::default(),
            seperator_style: Style::new(Color::White).dimmed(),
            url_style: Style::new(Color::Blue),
            warning_style: Style::new(Color::Yellow),
        }
    }

    pub fn action_style(&self) -> &Style {
        &self.action_style
    }

    pub fn action_paint<T>(&self, item: T) -> Paint<T> {
        self.action_style.paint(item)
    }

    pub fn arguments_style(&self) -> &Style {
        &self.arguments_style
    }

    pub fn arguments_paint<T>(&self, item: T) -> Paint<T> {
        self.arguments_style.paint(item)
    }

    pub fn command_style(&self) -> &Style {
        &self.command_style
    }

    pub fn command_paint<T>(&self, item: T) -> Paint<T> {
        self.command_style.paint(item)
    }

    pub fn error_style(&self) -> &Style {
        &self.error_style
    }

    pub fn error_paint<T>(&self, item: T) -> Paint<T> {
        self.error_style.paint(item)
    }

    pub fn header_prefix(&self) -> &str {
        self.header_prefix
    }

    pub fn header_prefix_style(&self) -> &Style {
        &self.header_prefix_style
    }

    pub fn header_prefix_paint<T>(&self, item: T) -> Paint<T> {
        self.header_prefix_style.paint(item)
    }

    pub fn output_style(&self) -> &Style {
        &self.output_style
    }

    pub fn output_paint<T>(&self, item: T) -> Paint<T> {
        self.output_style.paint(item)
    }

    pub fn seperator_style(&self) -> &Style {
        &self.seperator_style
    }

    pub fn seperator_paint<T>(&self, item: T) -> Paint<T> {
        self.seperator_style.paint(item)
    }

    pub fn url_style(&self) -> &Style {
        &self.url_style
    }

    pub fn url_paint<T>(&self, item: T) -> Paint<T> {
        self.url_style.paint(item)
    }

    pub fn warning_style(&self) -> &Style {
        &self.warning_style
    }

    pub fn warning_paint<T>(&self, item: T) -> Paint<T> {
        self.warning_style.paint(item)
    }
}

/*
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
}*/
