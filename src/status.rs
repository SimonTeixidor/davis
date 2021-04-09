use crate::ansi::{FormattedString, Style};
use crate::error::Error;
use crate::table::{Table, TableRow};
use std::time::Duration;

pub fn status(c: &mut mpd::Client, disable_formatting: bool) -> Result<(), Error> {
    let song = c.currentsong()?;
    let status = c.status()?;

    let mut table_rows = vec![];

    if let Some(song) = song.as_ref() {
        table_rows.push(table_row("Song", &*song.file));
    }

    let update_status = status
        .updating_db
        .map(|update_id| format!("DB update #{} in progress.", update_id));
    if let Some(status) = update_status.as_ref() {
        table_rows.push(table_row("Update", &*status));
    }

    let time = status.time.map(|(current, total)| {
        format!(
            "{}/{} ({:2}%)",
            duration_format(&current),
            duration_format(&total),
            (100. * current.as_secs_f64() / total.as_secs_f64()) as u32
        )
    });

    if let Some(time) = time.as_ref() {
        table_rows.push(table_row("Time", &*time));
    }

    let state = match status.state {
        mpd::State::Play => "playing",
        mpd::State::Pause => "paused",
        mpd::State::Stop => "Stopped",
    };
    table_rows.push(table_row("State", state));

    let queue_position = status.song.map(|s| format!("{}", 1 + s.pos));
    if let Some(pos) = queue_position.as_ref() {
        table_rows.push(table_row("Queue Position", &*pos));
    }
    let volume = format!("{}%", status.volume);
    table_rows.push(table_row("Volume", &*volume));
    table_rows.push(table_row("Repeat", bool_on_off(status.repeat)));
    table_rows.push(table_row("Random", bool_on_off(status.random)));
    table_rows.push(table_row("Single", bool_on_off(status.single)));
    table_rows.push(table_row("Consume", bool_on_off(status.consume)));
    println!(
        "{}",
        Table {
            rows: &*table_rows,
            disable_formatting
        }
    );
    Ok(())
}

// Table row with bold key and normal value
fn table_row<'a>(key: &'a str, val: &'a str) -> TableRow<'a> {
    TableRow::new(
        FormattedString::new(key).style(Style::Bold),
        FormattedString::new(val),
    )
}

fn bool_on_off(b: bool) -> &'static str {
    if b {
        "on"
    } else {
        "off"
    }
}

fn duration_format(d: &Duration) -> String {
    format!("{:02}:{:02}", d.as_secs() / 60, d.as_secs() % 60)
}
