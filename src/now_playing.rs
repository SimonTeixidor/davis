use crate::ansi::{FormattedString, Style};
use crate::config::{Config, Tag};
use crate::error::Error;
use crate::table::{Table, TableRow};
use crate::tags::Tags;

pub fn now_playing(client: &mut mpd::Client, conf: &Config) -> Result<(), Error> {
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
                    TableRow::new(vec![
                        FormattedString::new(&*label.as_ref().unwrap_or(tag)).style(Style::Bold),
                        FormattedString::new(*value),
                    ])
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

fn classical_work_description(tags: &Tags) -> Option<String> {
    let title = tags
        .joined(&["MOVEMENTNUMBER", "MOVEMENT"], ". ")
        .or_else(|| tags.get_option_joined("TITLE"))?;

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
