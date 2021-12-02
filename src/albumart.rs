use crate::error::{Error, WithContext};
use mpd::client::Client;
use std::fs::File;
use std::io::Write;
use std::process::exit;

pub fn fetch(client: &mut Client, song_path: Option<&str>, output: &str) -> Result<(), Error> {
    client.binarylimit(4_000_000)?;

    let album_art = if let Some(path) = song_path {
        client.albumart(&*path)?
    } else if let Some(song) = client.currentsong()? {
        client.albumart(&*song.file)?
    } else {
        println!("No song specified and no song is currently playing.");
        exit(1);
    };

    if "-" == output {
        std::io::copy(&mut &*album_art, &mut std::io::stdout().lock())
            .context("writing to albumart to stdout")?;
    } else {
        File::create(output)
            .context("creating albumart file")?
            .write_all(&*album_art)
            .context("writing albumart to file")?;
    };
    Ok(())
}
