use crate::ansi::{FormattedString, Style};
use crate::config::COLUMN_WIDTH;
use crate::error::{Error, WithContext};
use crate::table::{Table, TableRow};
use crate::tags::Tags;
use crate::terminal_dimensions;
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

pub fn now_playing(client: &mut mpd::Client) -> Result<(), Error> {
    let winsize = terminal_dimensions::terminal_size();
    let width = COLUMN_WIDTH as usize;
    let char_width = if winsize.ws_col != 0 && winsize.ws_xpixel != 0 {
        winsize.ws_xpixel / winsize.ws_col
    } else {
        10
    };
    let image_width = width as u32 * char_width as u32;

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

    match fetch_albumart(&song, client, image_width) {
        Ok(albumart) => match std::io::stdout().lock().write_all(&*albumart) {
            Err(e) => println!("Failed to write album art to stdout: {}", e),
            Ok(_) => (),
        },
        Err(e) => println!("Failed to fetch album art: {}", e),
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

fn fetch_albumart(song: &Song, client: &mut Client, width: u32) -> Result<Vec<u8>, Error> {
    let path = albumart_cache_path(song);

    create_dir_all(path.parent().expect("Albumart path has no parent?!"))
        .context("Failed to create dir for albumart cache.")?;

    if !path.exists() {
        let albumart = client.albumart(song)?;
        let mut file = File::create(&path).context("Failed to create albumart file.")?;
        file.write_all(&*albumart)
            .context("Failed to write albumart to file.")?;
    }

    let img = image::io::Reader::open(path)
        .context("Couldn't open albumart_path.")?
        .with_guessed_format()
        .context("Couldn't guess format of album art.")?
        .decode()?;
    let sixel = sixel::to_sixel(width, &img, 1024).context("generating sixel")?;
    Ok(sixel)
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
