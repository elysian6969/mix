#![allow(dead_code)]

pub use async_trait::async_trait;
use core::fmt::Arguments;

pub use crate::shell::Shell;
pub use crate::theme::Theme;

mod shell;
mod theme;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T = (), E = Error> = std::result::Result<T, E>;

#[async_trait(?Send)]
pub trait AsyncWrite {
    async fn write_str(&self, string: &str) -> Result;

    async fn write_char(&self, character: char) -> Result {
        self.write_str(character.encode_utf8(&mut [0; 4])).await
    }

    async fn write_fmt<'a>(&'a self, args: Arguments<'a>) -> Result {
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

#[async_trait(?Send)]
pub trait AsyncDisplay<W: AsyncWrite> {
    async fn fmt(&self, fmt: &W) -> Result;
}

#[macro_export]
macro_rules! write {
    ($dst:expr, $($arg:tt)*) => ({
        use $crate::AsyncWrite;

        $dst.write_fmt(::core::format_args!($($arg)*)).await
    });
}

#[macro_export]
macro_rules! writeln {
    ($dst:expr $(,)?) => ({
        $crate::write!($dst, "\n")
    });
    ($dst:expr, $($arg:tt)*) => ({
        use $crate::AsyncWrite;

        $dst.write_fmt(::core::format_args_nl!($($arg)*)).await
    });
}

#[macro_export]
macro_rules! header {
    ($dst:expr, $fmt:expr, $($arg:tt)*) => ({
        $crate::writeln!(
            $dst,
            ::core::concat!("{}", $fmt),
            $dst.theme().header_prefix(),
            $($arg)*
        )
    });
}


#[async_trait(?Send)]
impl AsyncDisplay<Shell> for url::Url {
    async fn fmt(&self, fmt: &Shell) -> Result<()> {
        write!(fmt, "{}", fmt.theme().url_paint(self))
    }
}
