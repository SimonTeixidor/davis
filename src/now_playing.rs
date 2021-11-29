use crate::ansi::{is_dumb_terminal, FormattedString, Style};
use crate::config::{Config, Tag};
use crate::error::{Error, WithContext};
use crate::filecache;
use crate::table::{Table, TableRow};
use crate::tags::Tags;
use mpd::{Client, Song};
use std::io::Write;
use std::ops::Add;
use std::path::PathBuf;

pub fn now_playing(client: &mut mpd::Client, cache: bool, conf: &Config) -> Result<(), Error> {
    let song = match client.currentsong()? {
        None => {
            println!("Not playing.");
            return Ok(());
        }
        Some(s) => s,
    };

    let tags = Tags::from_song_and_raw_comments(
        &song,
        client
            .readcomments(&*song.file)?
            .collect::<Result<_, _>>()?,
    );

    if !is_dumb_terminal() {
        match fetch_albumart(&song, client, cache) {
            Ok(albumart) => {
                use std::process::Command;
                Command::new("pica")
                    .args(["-w", "500"])
                    .arg(albumart)
                    .spawn()
                    .unwrap()
                    .wait()
                    .unwrap();
            }
            Err(e) => log::error!("Failed to fetch album art: {}", e),
        }
    }

    let table_rows = conf
        .tags
        .iter()
        .map(|Tag { tag, label }| {
            tags.get_option(&*tag)
                .as_ref()
                .iter()
                .flat_map(|values| {
                    values.iter().map(|value| {
                        TableRow::new(vec![
                            FormattedString::new(&*label.as_ref().unwrap_or(tag))
                                .style(Style::Bold),
                            FormattedString::new(*value),
                        ])
                    })
                })
                .collect::<Vec<_>>()
        })
        .flat_map(|v| v.into_iter())
        .collect::<Vec<_>>();

    println!("{}", header(&tags));
    println!("{}", Table { rows: &*table_rows },);
    Ok(())
}

fn header(tags: &Tags) -> String {
    classical_work_description(tags)
        .or_else(|| popular_music_title(tags))
        .unwrap_or_else(|| "".to_string())
}

fn fetch_albumart(song: &Song, client: &mut Client, cache: bool) -> Result<PathBuf, Error> {
    let cache_key = song.file.rsplit('/').skip(1).fold(String::new(), Add::add);

    filecache::cache(
        &*cache_key,
        move |f| {
            client.binarylimit(4_000_000)?;
            let albumart = client.albumart(&*song.file)?;
            f.write_all(&*albumart)
                .context("writing album art to cache")?;
            Ok(())
        },
        !cache,
    )
}

fn classical_work_description(tags: &Tags) -> Option<String> {
    let title = tags
        .get_option_joined("MOVEMENTNUMBER")
        .and_then(|n| {
            tags.get_option_joined("MOVEMENT")
                .map(|m| format!("{}. {}", n, m))
        })
        .or_else(|| tags.get_option_joined("TITLE").map(String::from))?;

    Some(format!(
        "{}\n{}\n{}\n",
        FormattedString::new(&*tags.get_option_joined("COMPOSER")?).style(Style::Bold),
        FormattedString::new(&*tags.get_option_joined("WORK")?).style(Style::Bold),
        FormattedString::new(&*title).style(Style::Bold)
    ))
}

fn popular_music_title(tags: &Tags) -> Option<String> {
    Some(format!(
        "{}\n{}\n",
        FormattedString::new(&*tags.get_option_joined("ARTIST")?).style(Style::Bold),
        FormattedString::new(&*tags.get_option_joined("TITLE")?).style(Style::Bold),
    ))
}
