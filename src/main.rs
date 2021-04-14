use clap::Clap;
use mpd::lsinfo::LsInfoResponse;
use mpd::Client;
use mpd::Song;
use std::env;
use std::ffi::OsString;
use std::net::TcpStream;
use std::process::Command;

mod albumart;
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
            println!("{}", e);
            std::process::exit(1);
        }
    }
}

fn try_main() -> Result<(), Error> {
    let conf = config::get_config()?;
    let opts = parse_args();
    let mpd_host = match (opts.host, config::mpd_host_env_var()) {
        (_, Some(host)) => host,
        (Some(host), _) => {
            if let Some(host) = conf.hosts.iter().find(|h| h.label.as_ref() == Some(&host)) {
                host.host.clone()
            } else {
                host
            }
        }
        _ => conf.default_mpd_host(),
    };

    let mpd_host_str = format!("{}:6600", mpd_host);

    let mut c = Client::new(TcpStream::connect(&mpd_host_str).context("connecting to MPD")?)?;

    match opts.subcommand.expect("no subcommand, this is a bug.") {
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
        SubCommand::Queue { group } => queue::print(
            c.queue()?,
            c.currentsong()?,
            group.unwrap_or(conf.grouped_queue),
        )?,
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
        SubCommand::Albumart { song_path, output } => {
            albumart::fetch_albumart(&mut c, song_path.as_ref().map(|s| &**s), &*output)?;
        }
        SubCommand::Custom(args) => {
            Command::new(&args[0])
                .env("MPD_HOST", mpd_host)
                .args(&args[1..])
                .spawn()
                .context("spawning child process")?
                .wait()
                .context("waiting for child process")?;
        }
    }
    Ok(())
}

fn parse_args() -> Opts {
    let args = env::args_os().collect::<Vec<_>>();
    let mut opts = Opts::parse_from(args);
    if opts.subcommand.is_none() {
        opts.subcommand = Some(SubCommand::Current {
            no_cache: false,
            no_format: false,
        });
    }

    match &opts.subcommand {
        Some(SubCommand::Custom(v)) => {
            let mut v = v.clone();
            if let Some(subcommand) = find_subcommand(&*v[0]) {
                v[0] = subcommand.as_os_str().to_owned();
                Opts {
                    host: opts.host,
                    subcommand: Some(SubCommand::Custom(v)),
                }
            } else {
                eprintln!("{} is not a known subcommand.", v[0].to_string_lossy());
                std::process::exit(1);
            }
        }
        _ => opts,
    }
}

fn trim_path(path: &str) -> &str {
    path.trim_end_matches('/')
}

#[derive(Clap)]
#[clap(author = clap::crate_authors!(), version = clap::crate_version!())]
struct Opts {
    #[clap(long, short)]
    /// The MPD server, can be specified using IP/hostname, or a label defined in the config file.
    host: Option<String>,
    #[clap(subcommand)]
    subcommand: Option<SubCommand>,
}

#[derive(Clap)]
enum SubCommand {
    /// Display the currently playing song.
    Current {
        #[clap(long, short)]
        /// Fetch new album art from MPD, ignoring any cached images..
        no_cache: bool,
        #[clap(long, short)]
        /// Print only plain text in a key=value format.
        plain: bool,
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
        #[clap(long, short)]
        /// Group the queue by artist/album, or composer/work group is true. This overrides
        /// grouped_queue from the config file.
        group: Option<bool>,
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
        #[clap(long, short)]
        /// Print only plain text in a key=value format.
        plain: bool,
    },
    /// Update the MPD database.
    Update,
    /// Display MPD status.
    Status {
        #[clap(long, short)]
        /// Print only plain text in a key=value format.
        plain: bool,
    },
    /// Download albumart for track.
    Albumart {
        /// The song to fetch albumart for. Will use currently playing song if not provided.
        song_path: Option<String>,
        #[clap(long, short)]
        /// Path to a file where the image will be stored. The image can be written to stdout by
        /// supplying a '-' character.
        output: String,
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
