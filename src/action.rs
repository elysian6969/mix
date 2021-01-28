use crate::args::{Install, Remove, Update};

mod fetch;
mod sync;

pub use fetch::fetch;
pub use sync::sync;

pub async fn install(install: Install) -> anyhow::Result<()> {
    Ok(())
}

pub async fn remove(remove: Remove) -> anyhow::Result<()> {
    Ok(())
}

pub async fn update(update: Update) -> anyhow::Result<()> {
    Ok(())
}
