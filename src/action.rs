use crate::args::{Add, Del, Update};

mod depends;
mod fetch;
mod sync;

pub use depends::depends;
pub use fetch::fetch;
pub use sync::sync;

pub async fn add(add: Add) -> anyhow::Result<()> {
    Ok(())
}

pub async fn del(Del: Del) -> anyhow::Result<()> {
    Ok(())
}

pub async fn update(update: Update) -> anyhow::Result<()> {
    Ok(())
}
