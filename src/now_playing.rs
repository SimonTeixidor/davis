use crate::ansi::{FormattedString, Style};
use crate::error::{Error, WithContext};
use crate::table::{Table, TableRow};
use crate::tags::Tags;
use libc::winsize;
use mpd::{Client, Song};
use std::env;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::ops::Add;
use std::path::PathBuf;

static INTERESTING_TAGS: &[(&str, &str)] = &[
    ("COMPOSER", "Composer"),
    ("WORK", "Work"),
    ("OPUS", "Opus"),
    ("CONDUCTOR", "Conductor"),
    ("ENSEMBLE", "Ensemble"),
    ("PERFORMER", "Performer"),
    ("RECORDINGLOCATION", "Location"),
    ("RECORDINGDATE", "Recording date"),
    ("LABEL", "Label"),
    ("RATING", "Rating"),
];

pub fn now_playing(client: &mut mpd::Client, winsize: &winsize) -> Result<(), Error> {
    let char_width = (winsize.ws_xpixel / winsize.ws_col) as usize;
    let width = winsize.ws_col.min(50) as usize;
    let image_width = (width * char_width) as u64;
    let song = match client.currentsong()? {
        None => {
            println!("Not playing.");
            return Ok(());
        }
        Some(s) => s,
    };

    let tags = Tags::from_song_and_raw_comments(
        &song,
        client.readcomments(&song)?.collect::<Result<_, _>>()?,
    );
    if let Ok(albumart_path) = fetch_albumart(&song, client) {
        let encoder = sixel::encoder::Encoder::new()?;
        encoder.set_width(sixel::optflags::SizeSpecification::Pixel(image_width))?;
        encoder.encode_file(&albumart_path)?;
    }

    println!("{}", header(&tags, width));

    let table_rows = INTERESTING_TAGS
        .iter()
        .map(|(tag, label)| {
            tags.get_option(&*tag)
                .as_ref()
                .iter()
                .flat_map(|values| {
                    values.iter().map(|value| {
                        TableRow::new(
                            FormattedString::new(&*label).style(Style::Bold),
                            FormattedString::new(&*value),
                        )
                    })
                })
                .collect::<Vec<_>>()
        })
        .flat_map(|v| v.into_iter())
        .collect::<Vec<_>>();
    println!("{}", Table(&*table_rows));
    Ok(())
}

fn header(tags: &Tags, width: usize) -> String {
    classical_work_description(tags, width)
        .or_else(|| popular_music_title(tags, width))
        .unwrap_or_else(|| "".to_string())
}

fn fetch_albumart(song: &Song, client: &mut Client) -> Result<PathBuf, Error> {
    let path = albumart_cache_path(song);

    create_dir_all(path.parent().expect("Albumart path has no parent?!"))
        .context("Failed to create dir for albumart cache.")?;

    if !path.exists() {
        let albumart = client.albumart(song)?;
        let mut file = File::create(&path).context("Failed to create albumart file.")?;
        file.write_all(&*albumart)
            .context("Failed to write albumart to file.")?;
    }
    Ok(path)
}

fn albumart_cache_path(song: &Song) -> PathBuf {
    let mut albumart_cache_path = env::temp_dir();
    albumart_cache_path.push("davis/album_art");
    let mut image_path = albumart_cache_path;
    let cache_key = song.file.rsplit('/').skip(1).fold(String::new(), Add::add);
    image_path.push(cache_key);
    image_path
}

fn classical_work_description(tags: &Tags, width: usize) -> Option<String> {
    let title = tags
        .get_option_joined("MOVEMENTNUMBER")
        .and_then(|n| {
            tags.get_option_joined("MOVEMENT")
                .map(|m| format!("{}. {}", n, m))
        })
        .or(tags.get_option_joined("TITLE").map(String::from))?;

    Some(format!(
        "{}\n{}\n{}\n",
        FormattedString::new(&*textwrap::fill(
            &*tags.get_option_joined("COMPOSER")?,
            width
        ))
        .style(Style::Bold),
        FormattedString::new(&*textwrap::fill(&*tags.get_option_joined("WORK")?, width))
            .style(Style::Bold),
        FormattedString::new(&*title).style(Style::Bold),
    ))
}

fn popular_music_title(tags: &Tags, width: usize) -> Option<String> {
    Some(format!(
        "{}\n{}\n",
        FormattedString::new(&*textwrap::fill(&*tags.get_option_joined("ARTIST")?, width))
            .style(Style::Bold),
        FormattedString::new(&*tags.get_option_joined("TITLE")?).style(Style::Bold),
    ))
}
