use crate::{Error, WithContext};
use std::env;
use std::fs::{create_dir_all, remove_file, File};
use std::path::PathBuf;

pub fn cache<F: FnMut(&mut File) -> Result<(), Error>>(
    name: &str,
    mut task: F,
    ignore_existing: bool,
) -> Result<File, Error> {
    let home = env::var("HOME").expect("$HOME was not set!");
    let mut cache_path: PathBuf = [&*home, ".cache", "davis", "albumart"].iter().collect();

    log::trace!("Creating cache path for albumart: {:?}", &cache_path);
    create_dir_all(&cache_path).context("creating dir for albumart cache")?;

    cache_path.push(name);

    log::trace!("Looking up album art file: {:?}", &cache_path);

    if !cache_path.exists() || ignore_existing {
        log::trace!(
            "Album art file does not exist in cache, creating: {:?}",
            &cache_path
        );
        let mut file = File::create(&cache_path).context("creating albumart file")?;
        match task(&mut file) {
            Ok(_) => (),
            Err(e) => {
                log::trace!(
                    "Failed to generate albumart file, removing file from cache dir: {:?}",
                    &cache_path
                );
                remove_file(&cache_path).context("removing corrupt albumart cache file")?;
                return Err(e);
            }
        }
    } else {
        log::trace!("Found album art file in cache: {:?}", &cache_path);
    }
    Ok(File::open(cache_path).context("opening albumart cache file")?)
}
