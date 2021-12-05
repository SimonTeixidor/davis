use crate::ansi::{FormattedString, Style};
use crate::config::{Config, Tag};
use crate::error::Error;
use crate::table::{Row, Table};
use crate::tags::Tags;
use mpdrs::Song;

pub fn now_playing(client: &mut mpdrs::Client, conf: &Config) -> Result<(), Error> {
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

    let table_rows = conf
        .tags
        .iter()
        .map(|Tag { tag, label }| {
            tags.get(&*tag)
                .iter()
                .map(|value| {
                    Row::new(vec![
                        FormattedString::new(&*label.as_ref().unwrap_or(tag)).style(Style::Bold),
                        FormattedString::new(*value),
                    ])
                })
                .collect::<Vec<_>>()
        })
        .flat_map(IntoIterator::into_iter)
        .collect::<Vec<_>>();

    println!("{}", header(&song, &tags));
    if !table_rows.is_empty() {
        println!("\n{}", Table { rows: &*table_rows },);
    }
    Ok(())
}

fn header(song: &Song, tags: &Tags) -> String {
    classical_work_description(tags)
        .or_else(|| popular_music_title(song))
        .unwrap_or_else(|| song.file.clone())
}

fn classical_work_description(tags: &Tags) -> Option<String> {
    let title = tags
        .joined(&["MOVEMENTNUMBER", "MOVEMENT"], ". ")
        .or_else(|| tags.get_option_joined("TITLE"))?;

    Some(format!(
        "{}\n{}\n{}",
        FormattedString::new(&*tags.get_option_joined("COMPOSER")?).style(Style::Bold),
        FormattedString::new(&*tags.get_option_joined("WORK")?).style(Style::Bold),
        FormattedString::new(&*title).style(Style::Bold)
    ))
}

fn popular_music_title(song: &Song) -> Option<String> {
    Some(format!(
        "{}\n{}",
        FormattedString::new(song.artist.as_deref()?).style(Style::Bold),
        FormattedString::new(&*song.title.as_deref()?).style(Style::Bold),
    ))
}
