use mpd::lsinfo::LsInfoResponse;
use mpd::Client;
use mpd::Song;
use std::net::TcpStream;

mod ansi;
mod config;
mod error;
mod filecache;
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
            match e {
                pico_args::Error::ArgumentParsingFailed { cause } => println!("{}", cause),
                _ => println!("{}", e),
            }
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
    let mut c = Client::new(TcpStream::connect(mpd_host()).context("Failed to connect to MPD.")?)?;
    match subcommand {
        SubCommand::NowPlaying => now_playing::now_playing(&mut c)?,
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
        SubCommand::Search(search) => {
            for song in c.search(&search.to_query(), None)? {
                println!("{}", song.file);
            }
        }
        SubCommand::List { tag, search } => {
            let query = search
                .as_ref()
                .map(|s| s.to_query())
                .unwrap_or(mpd::Query::Filters(mpd::FilterQuery::new()));

            for val in c.list(&mpd::Term::Tag(tag.into()), &query)? {
                println!("{}", val);
            }
        }
        SubCommand::ReadComments(p) => {
            let table_rows = c
                .readcomments(&p as &dyn AsRef<str>)?
                .collect::<Result<Vec<_>, _>>()?;
            let table_rows = table_rows
                .iter()
                .map(|(k, v)| {
                    table::TableRow::new(
                        ansi::FormattedString::new(&k).style(ansi::Style::Bold),
                        ansi::FormattedString::new(&v),
                    )
                })
                .collect::<Vec<_>>();
            println!("{}", table::Table(&*table_rows));
        }
        SubCommand::Update => {
            c.update()?;
        }
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
        Some("search") => Ok(SubCommand::Search(parse_search(pargs)?)),
        Some("list") => Ok(SubCommand::List {
            tag: pargs.free_from_str()?,
            search: parse_search(pargs).ok(),
        }),
        Some("readcomments") => Ok(SubCommand::ReadComments(pargs.free_from_str()?)),
        Some("update") => Ok(SubCommand::Update),
        None => Err(pico_args::Error::ArgumentParsingFailed {
            cause: format!("Missing subcommand"),
        }),
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
    Search(SearchType),
    List {
        tag: String,
        search: Option<SearchType>,
    },
    ReadComments(String),
    Update,
}

enum SearchType {
    Expr(String),
    TagValPairs(Vec<(String, String)>),
}

impl SearchType {
    fn to_query(&self) -> mpd::Query {
        match self {
            SearchType::TagValPairs(pairs) => {
                let mut query = mpd::FilterQuery::new();
                for (k, v) in pairs {
                    query.and(mpd::Term::Tag(k.into()), v);
                }
                mpd::Query::Filters(query)
            }
            SearchType::Expr(s) => mpd::Query::Expression(s.clone()),
        }
    }
}

fn parse_search(mut pargs: pico_args::Arguments) -> Result<SearchType, pico_args::Error> {
    match pargs.opt_value_from_str("--expr")? {
        Some(s) => Ok(SearchType::Expr(s)),
        None => {
            let remaining = pargs.finish();
            let remaining = remaining
                .iter()
                .map(|o| o.to_str())
                .collect::<Option<Vec<_>>>()
                .ok_or(pico_args::Error::NonUtf8Argument)?;

            if remaining.len() % 2 != 0 {
                return Err(pico_args::Error::ArgumentParsingFailed { cause : "Number of arguments to search was odd. Search expects key-value pairs, or an --expr option.".to_string() });
            }

            Ok(SearchType::TagValPairs(
                remaining
                    .chunks_exact(2)
                    .map(|v| (v[0].to_string(), v[1].to_string()))
                    .collect(),
            ))
        }
    }
}

fn mpd_host() -> String {
    let mpd_host = std::env::var("MPD_HOST");
    let mpd_host = mpd_host.as_ref().map(|s| &**s).unwrap_or("127.0.0.1");
    format!("{}:6600", mpd_host)
}

static HELP: &str = "\
USAGE:
  davis current             Display currently playing song
  davis pause               Pause playback
  davis play                Start playback
  davis toggle              Toggle playback
  davis ls [path]           List files in path
  davis clear               Clear the queue (and stop playback)
  davis next                Start playing next song on the queue
  davis prev                Start playing next previous song on the queue
  davis stop                Stop playback
  davis add path            Add path to queue
  davis load name           Replace queue with playlist
  davis queue               Display the current queue
  davis search --expr expr  Find tracks matching expr
  davis search --key val    Find tracks by sub-string search
  davis list [tag] [search] List all values for tag, for tracks matching search
  davis readcomments [path] List raw tags for song at path
  davis update              Update mpd database
";
