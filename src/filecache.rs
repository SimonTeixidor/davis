use crate::{Error, WithContext};
use std::env;
use std::fs::{create_dir_all, remove_file, File};
use std::path::PathBuf;

pub fn cache<F: FnMut(&mut File) -> Result<(), Error>>(
    name: &str,
    mut task: F,
) -> Result<File, Error> {
    let home = env::var("HOME").expect("$HOME was not set!");
    let mut cache_path: PathBuf = [&*home, ".cache", "davis", "albumart"].iter().collect();

    create_dir_all(&cache_path).context("Failed to create dir for albumart cache.")?;

    cache_path.push(name);

    if !cache_path.exists() {
        let mut file = File::create(&cache_path).context("Failed to create albumart file.")?;
        match task(&mut file) {
            Ok(_) => (),
            Err(e) => {
                remove_file(&cache_path).context("failed to remove corrupt albumart cache file")?;
                return Err(e);
            }
        }
    }
    Ok(File::open(cache_path).context("Failed to open albumart cache file.")?)
}
