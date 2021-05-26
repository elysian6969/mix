// tbh this needs to be a lib in general

use crossterm::style::Colorize;
use std::cell::RefCell;
use std::fmt::Display;
use tokio::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, Stdin, Stdout};

pub struct Shell {
    stdin: RefCell<BufReader<Stdin>>,
    stdout: RefCell<BufWriter<Stdout>>,
}

impl Shell {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn write_all(&self, bytes: &[u8]) -> io::Result<()> {
        self.stdout.borrow_mut().write_all(bytes).await
    }

    pub async fn flush(&self) -> io::Result<()> {
        self.stdout.borrow_mut().flush().await
    }

    pub async fn read_line(&self, buffer: &mut String) -> io::Result<usize> {
        self.stdin.borrow_mut().read_line(buffer).await
    }

    pub async fn confirm(&self, args: impl Display) -> io::Result<bool> {
        let text = format!("{args} {}/{} ", "y".green(), "n".red());
        self.write_all(text.as_bytes()).await?;

        let mut buffer = String::new();

        self.read_line(&mut buffer).await?;

        buffer.to_lowercase();

        Ok(buffer.starts_with('\n') || buffer.starts_with('y'))
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self {
            stdin: RefCell::new(BufReader::new(io::stdin())),
            stdout: RefCell::new(BufWriter::new(io::stdout())),
        }
    }
}

pub struct Text<D: ufmt::uDisplay> {
    display: D,
}

impl<D: ufmt::uDisplay> Text<D> {
    pub fn new(display: D) -> Self {
        Self { display }
    }

    pub async fn render(&self, shell: &Shell) -> crate::Result<()> {
        let buffer = ufmt::uformat!("{}", self.display)?;

        shell.write_all(buffer.as_bytes()).await?;
        shell.flush().await?;

        Ok(())
    }
}

use std::ops::RangeInclusive;

pub struct ProgressBar {
    range: RangeInclusive<f32>,
    value: f32,
    width: Option<f32>,
}

impl ProgressBar {
    pub fn new(range: RangeInclusive<f32>, value: f32) -> Self {
        Self {
            range,
            value,
            width: None,
        }
    }

    pub fn width(mut self, width: Option<f32>) -> Self {
        self.width = width;
        self
    }

    pub async fn render(&self, shell: &Shell) -> crate::Result<()> {
        let diff = self.value / self.range.end();
        let width = self.width.unwrap_or(50.0) * diff;
        let bar = "#".repeat(width as usize);

        shell.write_all(bar.as_bytes()).await?;
        shell.flush().await?;

        Ok(())
    }
}

pub enum Colour {
    None,
    Blue,
    Green,
    Magenta,
    Red,
    Yellow,
}

pub struct Line {
    buffer: String,
}

impl Line {
    pub fn new(display: impl ufmt::uDisplay, colour: Colour) -> Self {
        let mut buffer = String::new();

        unsafe {
            let display = colourise(display, colour);

            ufmt::uwrite!(&mut buffer, "{}", display).unwrap_unchecked();
        }

        Self { buffer }
    }

    pub fn append(&mut self, display: impl ufmt::uDisplay, colour: Colour) -> &mut Self {
        unsafe {
            let display = colourise(display, colour);

            ufmt::uwrite!(&mut self.buffer, " {}", display).unwrap_unchecked();
        }

        self
    }

    pub fn raw_append(&mut self, display: impl ufmt::uDisplay, colour: Colour) -> &mut Self {
        unsafe {
            let display = colourise(display, colour);

            ufmt::uwrite!(&mut self.buffer, "{}", display).unwrap_unchecked();
        }

        self
    }

    pub fn newline(&mut self) -> &mut Self {
        self.buffer.push('\n');
        self
    }

    pub async fn render(&self, shell: &Shell) -> crate::Result<()> {
        shell.write_all(self.buffer.as_bytes()).await?;
        shell.flush().await?;

        Ok(())
    }
}

// holy fuck this code is shit lmao
fn colourise(display: impl ufmt::uDisplay, colour: Colour) -> impl ufmt::uDisplay {
    let display = unsafe { ufmt::uformat!("{}", display).unwrap_unchecked() };

    match colour {
        Colour::None => display,
        Colour::Blue => display.blue().to_string(),
        Colour::Green => display.green().to_string(),
        Colour::Magenta => display.magenta().to_string(),
        Colour::Red => display.red().to_string(),
        Colour::Yellow => display.yellow().to_string(),
    }
}