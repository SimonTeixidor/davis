use crate::ansi::{FormattedString, Style};
use crate::error::Error;
use crate::tags::Tags;
use mpd::Song;

pub fn print(queue: Vec<Song>, current: Option<Song>, group: bool) -> Result<(), Error> {
    if group {
        print_grouped(queue, current)
    } else {
        print_flat(queue, current)
    }
}

fn print_grouped(queue: Vec<Song>, current: Option<Song>) -> Result<(), Error> {
    let mut group = None;
    for song in queue {
        let tags = Tags::from_song(&song);
        let new_group = if let (Some(work), Some(composer)) = (
            tags.get_option_joined("work"),
            tags.get_option_joined("composer"),
        ) {
            Some(format!("{} - {}", composer, work))
        } else if let (Some(album), Some(albumartist)) = (
            tags.get_option_joined("album"),
            tags.get_option_joined("albumartist"),
        ) {
            Some(format!("{} - {}", albumartist, album))
        } else {
            None
        };

        match new_group {
            Some(new_group) if group.as_ref() != Some(&new_group) => {
                group = Some(new_group.clone());
                println!("{}", FormattedString::new(&*new_group).style(Style::Bold))
            }
            _ => {}
        }

        let prefix = if group.is_some() { "  " } else { "" };

        let title = if let (Some(movementnumber), Some(movement)) = (
            tags.get_option_joined("movementnumber"),
            tags.get_option_joined("movement"),
        ) {
            format!("{}. {}", movementnumber, movement)
        } else if let (Some(artist), Some(title)) = (song.artist, song.title) {
            format!("{} - {}", artist, title)
        } else {
            song.file
        };
        let title = if current.as_ref().and_then(|s| s.place) == song.place {
            format!(
                "{}{}",
                prefix,
                FormattedString::new(&*title).style(Style::Bold)
            )
        } else {
            format!("{}{}", prefix, title)
        };
        println!("{}", title);
    }
    Ok(())
}

fn print_flat(queue: Vec<Song>, current: Option<Song>) -> Result<(), Error> {
    for song in queue {
        let title = match (song.artist, song.title) {
            (Some(artist), Some(title)) => format!("{} - {}", artist, title),
            _ => song.file,
        };
        let title = if current.as_ref().and_then(|s| s.place) == song.place {
            format!("{}", FormattedString::new(&*title).style(Style::Bold))
        } else {
            title
        };
        println!("{}", title);
    }
    Ok(())
}
