use crate::ansi::{FormattedString, Style};
use crate::table::{Row, Table};
use crate::tags::Tags;
use mpdrs::Song;

pub fn bold<S: AsRef<str>>(s: S) -> String {
    FormattedString::new(s.as_ref())
        .style(Style::Bold)
        .to_string()
}

fn header(song: &Song) -> Option<String> {
    let tags = Tags::from_song(song);
    tags.joined(&["work", "composer"], " - ")
        .or_else(|| tags.joined(&["album", "albumartist"], " - "))
        .or_else(|| tags.joined(&["album", "artist"], " - "))
}

struct QueueRow {
    is_current: bool,
    fields: Vec<String>,
}

impl QueueRow {
    fn to_table_row(&self) -> Row {
        Row::new(
            self.fields
                .iter()
                .map(|s| {
                    FormattedString::new(&*s).style(if self.is_current {
                        Style::Bold
                    } else {
                        Style::Default
                    })
                })
                .collect(),
        )
    }
}

fn print_table(rows: &[QueueRow]) {
    let table_rows = rows
        .iter()
        .map(QueueRow::to_table_row)
        .collect::<Vec<Row>>();
    println!("{}", Table { rows: &*table_rows });
}

pub fn print(queue: Vec<Song>, current: &Option<Song>) {
    let mut cur_header = None;
    let mut rows: Vec<QueueRow> = Vec::new();
    let max_pos = queue.len();
    let pos_width = (max_pos as f32).log10() as usize + 1;
    for (pos, song) in queue.into_iter().enumerate() {
        let pos = pos + 1;
        if let Some(h) = header(&song).filter(|h| Some(h) != cur_header.as_ref()) {
            if !rows.is_empty() {
                print_table(&*rows);
                rows.clear();
            }
            println!("{}", bold(&h));
            cur_header = Some(h);
        }

        let tags = Tags::from_song(&song);
        let mut fields = ["movementnumber", "movement"]
            .iter()
            .map(|s| tags.get_option_joined(s))
            .collect::<Option<Vec<String>>>()
            .or_else(|| song.title.clone().map(|t| vec![t]))
            .unwrap_or_else(|| vec![song.file.clone()]);
        fields.insert(0, format!("{: <width$}", pos, width = pos_width));
        rows.push(QueueRow {
            is_current: Some(&song) == current.as_ref(),
            fields,
        });
    }
    if !rows.is_empty() {
        print_table(&*rows);
    }
}
