use crate::args::{Add, Del, Up};

mod deps;
mod fetch;
mod sync;

pub use deps::deps;
pub use fetch::fetch;
pub use sync::sync;

pub async fn add(add: Add) -> anyhow::Result<()> {
    Ok(())
}

pub async fn del(del: Del) -> anyhow::Result<()> {
    Ok(())
}

pub async fn update(up: Up) -> anyhow::Result<()> {
    Ok(())
}
