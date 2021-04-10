use mpd::lsinfo::LsInfoResponse;
use mpd::Client;
use mpd::Song;
use std::env;
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::Command;

mod ansi;
mod config;
mod error;
mod filecache;
mod now_playing;
mod queue;
mod status;
mod subcommands;
mod table;
mod tags;
mod terminal_dimensions;

use ansi::{Colour, Colour::*, FormattedString, Style, Style::*};
use error::{Error, WithContext};

fn main() {
    // Don't crash with error message on broken pipes.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

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
    let conf = config::get_config()?;
    let subcommand: SubCommand = parse_args(&conf)?;
    let mpd_host = format!("{}:6600", conf.mpd_host);
    let mut c = Client::new(TcpStream::connect(&mpd_host).context("connecting to MPD.")?)?;
    match subcommand {
        SubCommand::NowPlaying {
            enable_image_cache,
            disable_formatting,
        } => now_playing::now_playing(&mut c, enable_image_cache, disable_formatting, &conf)?,
        SubCommand::Play => c.play()?,
        SubCommand::Pause => c.pause(true)?,
        SubCommand::Toggle => c.toggle_pause()?,
        SubCommand::Ls(path) => {
            let path = path.as_ref().map(|s| trim_path(&*s)).unwrap_or("");
            for entry in c.lsinfo(path)? {
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
        SubCommand::Add(p) => c.add(&*trim_path(&*p))?,
        SubCommand::Load(p) => c.load(&*p, ..)?,
        SubCommand::Queue(grouped) => queue::print(c.queue()?, c.currentsong()?, grouped)?,
        SubCommand::Search(search) => {
            for song in c.search(&search.to_query(), None)? {
                println!("{}", song.file);
            }
        }
        SubCommand::List { tag, search } => {
            for val in c.list(&mpd::Term::Tag(&*tag), &search.to_query())? {
                println!("{}", val);
            }
        }
        SubCommand::ReadComments {
            file,
            disable_formatting,
        } => {
            let table_rows = c
                .readcomments(&*trim_path(&*file))?
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
            println!(
                "{:width$}",
                table::Table {
                    rows: &*table_rows,
                    disable_formatting
                },
                width = conf.width
            );
        }
        SubCommand::Update => {
            c.update()?;
        }
        SubCommand::Status { disable_formatting } => {
            status::status(&mut c, disable_formatting, conf.width)?
        }
        SubCommand::Custom(path) => {
            Command::new(path)
                .env("MPD_HOST", conf.mpd_host)
                .args(env::args().skip(2))
                .spawn()
                .context("spawning child process")?
                .wait()
                .context("waiting for child process")?;
        }
    }
    Ok(())
}

fn parse_args(conf: &config::Config) -> Result<SubCommand, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    let subcommand = pargs.subcommand()?;

    if pargs.contains(["-h", "--help"]) || subcommand.as_ref().filter(|s| *s == "help").is_some() {
        print!("{}", HELP);
        std::process::exit(0);
    }

    let disable_formatting = pargs.contains("--no-format");

    match subcommand
        .as_ref()
        .or(conf.default_subcommand.as_ref())
        .map(|s| &**s)
    {
        Some("current") => Ok(SubCommand::NowPlaying {
            enable_image_cache: !pargs.contains("--no-cache"),
            disable_formatting,
        }),
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
            search: parse_search(pargs)?,
        }),
        Some("readcomments") => Ok(SubCommand::ReadComments {
            file: pargs.free_from_str()?,
            disable_formatting,
        }),
        Some("update") => Ok(SubCommand::Update),
        Some("status") => Ok(SubCommand::Status { disable_formatting }),
        Some("help") => Ok(SubCommand::Status { disable_formatting }),
        None => Err(pico_args::Error::ArgumentParsingFailed {
            cause: format!("Missing subcommand"),
        }),
        Some(s) => {
            let mut subcommands = subcommands::find_subcommands();
            let command_name = format!("davis-{}", s);
            if let Some(path) = subcommands.remove(&command_name) {
                Ok(SubCommand::Custom(path))
            } else {
                Err(pico_args::Error::ArgumentParsingFailed {
                    cause: format!("unknown subcommand {}", s),
                })
            }
        }
    }
}

fn trim_path(path: &str) -> &str {
    path.trim_end_matches('/')
}

enum SubCommand {
    NowPlaying {
        enable_image_cache: bool,
        disable_formatting: bool,
    },
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
        search: SearchType,
    },
    ReadComments {
        file: String,
        disable_formatting: bool,
    },
    Update,
    Status {
        disable_formatting: bool,
    },
    Custom(PathBuf),
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
                    query.and(mpd::Term::Tag(&*k), v);
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

static HELP: &str = "\
USAGE:
  davis current --no-cache  Display currently playing song. If --no-cache is
                            specified, davis will fetch albumart from MPD
                            even if it exists in cache.
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
  davis status              Print current status
  davis help                Print this help text

OPTIONS:
  --no-format               Makes current, readcomments, and status commands
                            write unformatted key-value pairs separated by '='.
  --group                   Causes the queue command to print songs grouped
                            by their album and artist, or composer and work.
";
