use crate::ansi::{Colour, FormattedString, Style};
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

static INTERESTING_TAGS: &[[&str; 2]] = &[
    ["COMPOSER", "Composer"],
    ["WORK", "Work"],
    ["OPUS", "Opus"],
    ["CONDUCTOR", "Conductor"],
    ["ENSEMBLE", "Ensemble"],
    ["PERFORMER", "Performer"],
    ["RECORDINGLOCATION", "Location"],
    ["RECORDINGDATE", "Recording date"],
    ["LABEL", "Label"],
    ["RATING", "Rating"],
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

    let tags = Tags::from_song(&song, client)?;
    if let Ok(albumart_path) = fetch_albumart(&song, client) {
        let encoder = sixel::encoder::Encoder::new()?;
        encoder.set_width(sixel::optflags::SizeSpecification::Pixel(image_width))?;
        encoder.encode_file(&albumart_path)?;
    }

    println!("{}", header(&tags, width));

    for [tag, label] in INTERESTING_TAGS {
        if let Some(vals) = tags.get_option(tag) {
            println!(
                "{}:",
                FormattedString::new(label)
                    .colour(Colour::BrightCyan)
                    .style(Style::Bold)
            );
            for val in vals {
                for (i, line) in textwrap::fill(val, width - 4).lines().enumerate() {
                    println!("{}{}", if i == 0 { "  • " } else { "    " }, line);
                }
            }
        }
    }
    Ok(())
}

fn header(tags: &Tags, width: usize) -> String {
    let composer = tags.get_option_joined("composer");
    let work = tags.get_option_joined("work");
    let ensemble = tags.get_option_joined("ensemble");
    let movement = tags.get_option_joined("movementname");
    let movementnumber = tags.get_option_joined("movement");
    let conductor = tags.get_option_joined("conductor");
    let title = tags.get_option_joined("title");
    let artist = tags.get_option_joined("artist");

    if let Some(description) = classical_work_description(
        composer.as_ref().map(|s| &**s),
        work.as_ref().map(|s| &**s),
        ensemble.as_ref().map(|s| &**s),
        conductor.as_ref().map(|s| &**s),
        movement.as_ref().map(|s| &**s),
        movementnumber.as_ref().map(|s| &**s),
        title.as_ref().map(|s| &**s),
        width,
    ) {
        description
    } else if let (Some(artist), Some(title)) = (artist.as_ref(), title.as_ref()) {
        format!(
            "{}\n{}",
            FormattedString::new(&*textwrap::fill(artist, width)).style(Style::Bold),
            FormattedString::new(&*textwrap::fill(title, width)).style(Style::Bold)
        )
    } else {
        "".to_string()
    }
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

fn classical_work_description(
    composer: Option<&str>,
    work: Option<&str>,
    ensemble: Option<&str>,
    conductor: Option<&str>,
    movement: Option<&str>,
    movement_number: Option<&str>,
    title: Option<&str>,
    width: usize,
) -> Option<String> {
    let mut table_rows = vec![];

    if let Some(ensemble) = ensemble {
        table_rows.push(TableRow::new(
            FormattedString::new("performed by:")
                .colour(Colour::DarkWhite)
                .style(Style::Faint),
            FormattedString::new(ensemble)
                .colour(Colour::BrightMagenta)
                .style(Style::Bold),
        ));
    }

    if let Some(conductor) = conductor {
        table_rows.push(TableRow::new(
            FormattedString::new("under:")
                .colour(Colour::DarkWhite)
                .style(Style::Faint),
            FormattedString::new(conductor)
                .colour(Colour::DarkMagenta)
                .style(Style::Bold),
        ));
    };

    let title = movement_number
        .and_then(|n| movement.map(|m| format!("{}. {}", n, m)))
        .or(title.map(String::from))?;

    let mut result = format!(
        "{}\n{}\n{}\n",
        FormattedString::new(&*textwrap::fill(&*composer?, width)).style(Style::Bold),
        FormattedString::new(&*textwrap::fill(&*work?, width)).style(Style::Bold),
        FormattedString::new(&*title).style(Style::Bold),
    );

    if !table_rows.is_empty() {
        result += &*format!("{:width$}\n", Table(&*table_rows), width = width);
    }

    Some(result)
}
