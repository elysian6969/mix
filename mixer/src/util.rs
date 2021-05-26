use std::path::Path;
use tokio::{fs, io};
use tokio_stream::wrappers::ReadDirStream;

pub async fn read_dir(path: impl AsRef<Path>) -> io::Result<ReadDirStream> {
    Ok(ReadDirStream::new(fs::read_dir(path).await?))
}
