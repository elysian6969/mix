use git2::build::RepoBuilder;
use git2::{FetchOptions, RemoteCallbacks, Repository};
use std::path::Path;

fn fetch_options() -> FetchOptions<'static> {
    let mut callbacks = RemoteCallbacks::new();

    callbacks
        .sideband_progress(|message| {
            println!("{:?}", std::str::from_utf8(message));

            true
        })
        .transfer_progress(|progress| {
            println!("total_objects: {}", progress.total_objects());
            println!("indexed_objects: {}", progress.indexed_objects());
            println!("received_objects: {}", progress.received_objects());
            println!("local_objects: {}", progress.local_objects());
            println!("total_deltas: {}", progress.total_deltas());
            println!("indexed_deltas: {}", progress.indexed_deltas());
            println!("received_bytes: {}", progress.received_bytes());

            true
        })
        .update_tips(|message, old, new| {
            println!("{}", message);
            println!("{}", old);
            println!("{}", new);

            true
        });

    let mut fetch_options = FetchOptions::new();

    fetch_options.remote_callbacks(callbacks);
    fetch_options
}

pub async fn remote_fetch(repository: &Repository) -> anyhow::Result<()> {
    println!(" -> fetch");

    let mut fetch_options = fetch_options();

    repository
        .find_remote("origin")?
        .fetch(&["msster"], Some(&mut fetch_options), None)?;

    Ok(())
}

pub async fn clone(url: &str, path: &impl AsRef<Path>) -> anyhow::Result<()> {
    println!(" -> clone");

    let mut builder = RepoBuilder::new();
    let fetch_options = fetch_options();

    builder.fetch_options(fetch_options);

    builder.clone(&url, path.as_ref())?;

    Ok(())
}
