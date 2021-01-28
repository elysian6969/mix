use std::path::Path;
use tokio::process::Command;

/// Execute autoreconf
pub async fn autoreconf(work: &Path) -> anyhow::Result<()> {
    Command::new("autoreconf")
        .current_dir(&work)
        .spawn()?
        .wait()
        .await?;

    Ok(())
}

/// Execute bootstrap
pub async fn bootstrap(work: &Path) -> anyhow::Result<()> {
    Command::new("./bootstrap")
        .current_dir(&work)
        .spawn()?
        .wait()
        .await?;

    Ok(())
}

/// Execute configure
pub async fn configure(work: &Path) -> anyhow::Result<()> {
    Command::new("./configure")
        .current_dir(&work)
        .spawn()?
        .wait()
        .await?;

    Ok(())
}

/// Execute make
pub async fn make(work: &Path) -> anyhow::Result<()> {
    Command::new("make")
        .current_dir(&work)
        .spawn()?
        .wait()
        .await?;

    Ok(())
}

/// Execute make install
pub async fn make_install(work: &Path) -> anyhow::Result<()> {
    Command::new("make")
        .arg("install")
        .current_dir(&work)
        .spawn()?
        .wait()
        .await?;

    Ok(())
}
