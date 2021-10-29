use crate::ansi::{FormattedString, Style};
use crate::tags::Tags;
use mpd::Song;

pub fn bold<S: AsRef<str>>(s: S) -> String {
    FormattedString::new(s.as_ref())
        .style(Style::Bold)
        .to_string()
}

fn tags_joined(tags: &Tags, keys: &[&str], sep: &str) -> Option<String> {
    keys.iter()
        .map(|k| tags.get_option_joined(k))
        .collect::<Option<Vec<_>>>()
        .map(|v| v.join(sep))
}

fn header(song: &Song) -> Option<String> {
    let tags = Tags::from_song(song);
    tags_joined(&tags, &["work", "composer"], " - ")
        .or_else(|| tags_joined(&tags, &["album", "albumartist"], " - "))
        .or_else(|| tags_joined(&tags, &["album", "artist"], " - "))
}

pub fn print(queue: Vec<Song>, current: Option<Song>) {
    let mut cur_header = None;
    for (pos, song) in queue.into_iter().enumerate() {
        if let Some(h) = header(&song).filter(|h| Some(h) != cur_header.as_ref()) {
            println!("{}", bold(&h));
            cur_header = Some(h);
        }

        let pos = format!("{}\t", pos + 1);
        let tags = Tags::from_song(&song);
        let meta = tags_joined(&tags, &["movementnumber", "movement"], "\t")
            .or_else(|| song.title.clone())
            .unwrap_or_else(|| song.file.clone());

        if Some(&song) == current.as_ref() {
            println!("{} {}", bold(pos), bold(meta));
        } else {
            println!("{} {}", pos, meta);
        }
    }
}
