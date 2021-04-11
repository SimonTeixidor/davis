use clap::Clap;
use mpd::lsinfo::LsInfoResponse;
use mpd::Client;
use mpd::Song;
use std::env;
use std::ffi::OsString;
use std::net::TcpStream;
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

use error::{Error, WithContext};
use subcommands::find_subcommand;

fn main() {
    // Don't crash with error message on broken pipes.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    match try_main() {
        Ok(_) => (),
        Err(e) => {
            println!("{:?}", e);
            std::process::exit(1);
        }
    }
}

fn try_main() -> Result<(), Error> {
    let conf = config::get_config()?;
    let subcommand: SubCommand = parse_args(&conf);
    let mpd_host = format!("{}:6600", conf.mpd_host);
    let mut c = Client::new(TcpStream::connect(&mpd_host).context("connecting to MPD.")?)?;
    match subcommand {
        SubCommand::Current {
            no_cache,
            no_format,
        } => now_playing::now_playing(&mut c, !no_cache, no_format, &conf)?,
        SubCommand::Play => c.play()?,
        SubCommand::Pause => c.pause(true)?,
        SubCommand::Toggle => c.toggle_pause()?,
        SubCommand::Ls { path } => {
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
        SubCommand::Add { path } => c.add(&*trim_path(&*path))?,
        SubCommand::Load { path } => c.load(&*path, ..)?,
        SubCommand::Queue { group } => queue::print(c.queue()?, c.currentsong()?, group)?,
        SubCommand::Search { query } => {
            for song in c.search(&query.to_mpd_query(), None)? {
                println!("{}", song.file);
            }
        }
        SubCommand::List { tag, query } => {
            for val in c.list(&mpd::Term::Tag(&*tag), &query.to_mpd_query())? {
                println!("{}", val);
            }
        }
        SubCommand::ReadComments { file, no_format } => {
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
                    disable_formatting: no_format
                },
                width = conf.width
            );
        }
        SubCommand::Update => {
            c.update()?;
        }
        SubCommand::Status { no_format } => status::status(&mut c, no_format, conf.width)?,
        SubCommand::Custom(args) => {
            Command::new(&args[0])
                .env("MPD_HOST", conf.mpd_host)
                .args(&args[1..])
                .spawn()
                .context("spawning child process")?
                .wait()
                .context("waiting for child process")?;
        }
    }
    Ok(())
}

fn parse_args(conf: &config::Config) -> SubCommand {
    let mut args = env::args_os().collect::<Vec<_>>();
    match &conf.default_subcommand {
        Some(s) if args.len() == 1 => args.push(s.into()),
        _ => (),
    }
    match Opts::parse_from(args).subcommand {
        SubCommand::Custom(mut v) => {
            if let Some(subcommand) = find_subcommand(&*v[0]) {
                v[0] = subcommand.as_os_str().to_owned();
                SubCommand::Custom(v)
            } else {
                eprintln!("{} is not a known subcommand.", v[0].to_string_lossy());
                std::process::exit(1);
            }
        }
        s => s,
    }
}

fn trim_path(path: &str) -> &str {
    path.trim_end_matches('/')
}

#[derive(Clap)]
#[clap(author = clap::crate_authors!(), version = clap::crate_version!())]
struct Opts {
    #[clap(subcommand)]
    subcommand: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    /// Display the currently playing song.
    Current {
        #[clap(long)]
        /// Fetch new album art from MPD, ignoring any cached images..
        no_cache: bool,
        #[clap(long)]
        /// Print only plain text in a key=value format.
        no_format: bool,
    },
    /// Start playback.
    Play,
    /// Pause playback.
    Pause,
    /// Toggle between play/pause.
    Toggle,
    /// List items in path.
    Ls { path: Option<String> },
    /// Clear the current queue.
    Clear,
    /// Skip to next song in queue.
    Next,
    /// Go back to previous song in queue.
    Prev,
    /// Stop playback.
    Stop,
    /// Add items in path to queue.
    Add { path: String },
    /// Load playlist at path to queue.
    Load { path: String },
    /// Display the current queue.
    Queue {
        #[clap(long)]
        /// Group the queue by artist/album, or composer/work.
        group: bool,
    },
    /// Search the MPD database.
    Search {
        #[clap(flatten)]
        query: SearchQuery,
    },
    /// List all values for tag type, matching query.
    List {
        /// List values for this tag type
        tag: String,
        #[clap(flatten)]
        query: SearchQuery,
    },
    /// Read raw metadata tags for file.
    ReadComments {
        file: String,
        #[clap(long)]
        /// Print only plain text in a key=value format.
        no_format: bool,
    },
    /// Update the MPD database.
    Update,
    /// Display MPD status.
    Status {
        #[clap(long)]
        /// Print only plain text in a key=value format.
        no_format: bool,
    },
    #[clap(external_subcommand)]
    Custom(Vec<OsString>),
}

#[derive(Clap)]
struct SearchQuery {
    /// Either a single value with a search expression, or a sequence of key-value pairs.
    query: Vec<String>,
}

impl SearchQuery {
    fn to_mpd_query(&self) -> mpd::Query {
        if self.query.len() == 1 {
            mpd::Query::Expression(self.query[0].clone())
        } else {
            let mut query = mpd::FilterQuery::new();
            for slice in self.query.chunks(2) {
                query.and(mpd::Term::Tag(&*slice[0]), &*slice[1]);
            }
            mpd::Query::Filters(query)
        }
    }
}
