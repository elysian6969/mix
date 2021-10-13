use crate::{AsyncWrite, Result, Theme};
use async_trait::async_trait;
use core::fmt::{Arguments, Display};
use std::sync::Arc;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter, Stderr, Stdin, Stdout};
use tokio::sync::{RwLock, RwLockWriteGuard};

type BufErr = BufWriter<Stderr>;
type BufIn = BufReader<Stdin>;
type BufOut = BufWriter<Stdout>;

type Guard<'a, T> = RwLockWriteGuard<'a, T>;

#[derive(Debug)]
pub struct Shell {
    stderr: Arc<RwLock<BufErr>>,
    stdin: Arc<RwLock<BufIn>>,
    stdout: Arc<RwLock<BufOut>>,
    theme: Theme,
}

impl Shell {
    pub fn new() -> Self {
        // TODO: Replace with a single non-blocking handle to `/dev/tty`.
        let stderr = Arc::new(RwLock::new(BufWriter::new(io::stderr())));
        let stdin = Arc::new(RwLock::new(BufReader::new(io::stdin())));
        let stdout = Arc::new(RwLock::new(BufWriter::new(io::stdout())));
        let theme = Theme::new();

        Self {
            stderr,
            stdin,
            stdout,
            theme,
        }
    }

    async fn stderr_lock(&self) -> Guard<'_, BufErr> {
        self.stderr.write().await
    }

    async fn stderr_write(&self, bytes: &[u8]) -> io::Result<()> {
        self.stderr_lock().await.write_all(bytes).await?;

        Ok(())
    }

    async fn stderr_flush(&self) -> io::Result<()> {
        self.stderr_lock().await.flush().await?;

        Ok(())
    }

    async fn stdin_lock(&self) -> Guard<'_, BufIn> {
        self.stdin.write().await
    }

    async fn stdin_read(&self, bytes: &mut [u8]) -> io::Result<usize> {
        self.stdin_lock().await.read(bytes).await
    }

    async fn stdout_lock(&self) -> Guard<'_, BufOut> {
        self.stdout.write().await
    }

    async fn stdout_write(&self, bytes: &[u8]) -> io::Result<()> {
        self.stdout_lock().await.write_all(bytes).await?;

        Ok(())
    }

    async fn stdout_flush(&self) -> io::Result<()> {
        self.stdout_lock().await.flush().await?;

        Ok(())
    }
}

#[async_trait(?Send)]
impl AsyncWrite for Shell {
    async fn write_str(&self, string: &str) -> Result {
        self.stdout_write(string.as_bytes()).await?;

        Ok(())
    }

    async fn flush(&self) -> Result {
        self.stdout_flush().await?;

        Ok(())
    }
}
