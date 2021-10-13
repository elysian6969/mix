use async_trait::async_trait;
use core::fmt::Arguments;

pub use crate::shell::Shell;
pub use crate::theme::Theme;

mod shell;
mod theme;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub(crate) type Result<T = (), E = Error> = std::result::Result<T, E>;

#[async_trait(?Send)]
pub trait AsyncWrite {
    async fn write_str(&self, string: &str) -> Result;

    async fn write_char(&self, character: char) -> Result {
        self.write_str(character.encode_utf8(&mut [0; 4])).await
    }

    async fn write_fmt(&self, args: Arguments<'_>) -> Result {
        // TODO: Don't allocate a string.
        let mut buf = String::new();

        core::fmt::write(&mut buf, args)?;

        self.write_str(buf.as_str()).await?;

        Ok(())
    }

    async fn flush(&self) -> Result {
        Ok(())
    }
}

#[macro_export]
macro_rules! write {
    ($dst:expr, $($arg:tt)*) => ($dst.write_fmt(core::format_args!($($arg)*)))
}
