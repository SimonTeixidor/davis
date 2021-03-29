use mpd::lsinfo::LsInfoResponse;
use mpd::Client;
use mpd::Song;
use std::net::TcpStream;

mod ansi;
mod error;
mod now_playing;
mod queue;
mod table;
mod tags;
mod terminal_dimensions;

use ansi::{Colour, Colour::*, FormattedString, Style, Style::*};
use error::{Error, WithContext};

fn main() {
    match try_main() {
        Ok(_) => (),
        Err(Error::PicoError(e)) => {
            print_formatted("Failed to parse command line arguments.", BrightRed, Bold);
            println!();
            print_formatted("Caused by:", White, Bold);
            println!("{}", e);
            println!();
            print_formatted("Please consult the help page:", White, Bold);
            println!("{}", HELP);
            std::process::exit(1);
        }
        Err(e) => {
            println!("{:?}", e);
            std::process::exit(1);
        }
    }
}

fn print_formatted(s: &str, colour: Colour, style: Style) {
    println!("{}", FormattedString::new(s).style(style).colour(colour));
}

fn try_main() -> Result<(), Error> {
    let subcommand: SubCommand = parse_args()?;
    let mut c =
        Client::new(TcpStream::connect("127.0.0.1:6600").context("Failed to connect to MPD.")?)?;
    match subcommand {
        SubCommand::NowPlaying => {
            let winsize = terminal_dimensions::terminal_size();
            now_playing::now_playing(&mut c, &winsize).unwrap();
        }
        SubCommand::Play => c.play()?,
        SubCommand::Pause => c.pause(true)?,
        SubCommand::Toggle => c.toggle_pause()?,
        SubCommand::Ls(path) => {
            let path = path.as_ref().map(|s| s.trim_end_matches('/')).unwrap_or("");
            for entry in c.lsinfo(&path as &dyn AsRef<str>)? {
                match entry {
                    LsInfoResponse::Song(Song { file, .. }) => println!("{}", file),
                    LsInfoResponse::Directory { path, .. }
                    | LsInfoResponse::Playlist { path, .. } => println!("{}", path),
                }
            }
        }
        SubCommand::Clear => c.clear()?,
        SubCommand::Next => c.next()?,
        SubCommand::Prev => c.prev()?,
        SubCommand::Stop => c.stop()?,
        SubCommand::Add(p) => c.add(&p as &dyn AsRef<str>)?,
        SubCommand::Load(p) => c.load(p, ..)?,
        SubCommand::Queue(grouped) => queue::print(c.queue()?, c.currentsong()?, grouped)?,
    }
    Ok(())
}

fn parse_args() -> Result<SubCommand, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }

    match pargs.subcommand()?.as_ref().map(|s| &**s) {
        Some("current") => Ok(SubCommand::NowPlaying),
        Some("play") => Ok(SubCommand::Play),
        Some("pause") => Ok(SubCommand::Pause),
        Some("toggle") => Ok(SubCommand::Toggle),
        Some("ls") => Ok(SubCommand::Ls(pargs.opt_free_from_str()?)),
        Some("clear") => Ok(SubCommand::Clear),
        Some("next") => Ok(SubCommand::Next),
        Some("prev") => Ok(SubCommand::Prev),
        Some("stop") => Ok(SubCommand::Stop),
        Some("add") => Ok(SubCommand::Add(pargs.free_from_str()?)),
        Some("load") => Ok(SubCommand::Load(pargs.free_from_str()?)),
        Some("queue") => Ok(SubCommand::Queue(pargs.contains("--group"))),
        None => Ok(SubCommand::NowPlaying),
        Some(s) => Err(pico_args::Error::ArgumentParsingFailed {
            cause: format!("unknown subcommand {}", s),
        }),
    }
}

enum SubCommand {
    NowPlaying,
    Play,
    Pause,
    Toggle,
    Ls(Option<String>),
    Clear,
    Next,
    Prev,
    Stop,
    Add(String),
    Load(String),
    Queue(bool),
}

static HELP: &str = "\
USAGE:
  davis [current] Display currently playing song
  davis pause     Pause playback
  davis play      Start playback
  davis toggle    Toggle playback
  davis ls [path] List files in path
  davis clear     Clear the queue (and stop playback)
  davis next      Start playing next song on the queue
  davis prev      Start playing next previous song on the queue
  davis stop      Stop playback
  davis add path  Add path to queue
  davis load name Replace queue with playlist
  davis queue     Display the current queue
";
