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
mod logger;
mod now_playing;
mod queue;
mod seek;
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
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

fn try_main() -> Result<(), Error> {
    let opts = parse_args();
    let conf = config::get_config()?;

    let mpd_host = mpd_host(&opts, &conf);
    let mpd_host_str = format!("{}:6600", &mpd_host);

    let mut c = Client::new(TcpStream::connect(&mpd_host_str).context("connecting to MPD")?)?;

    match opts.subcommand.expect("no subcommand, this is a bug.") {
        SubCommand::Current { no_cache, plain } => {
            now_playing::now_playing(&mut c, !no_cache, plain, &conf)?
        }
        SubCommand::Play { position: Some(id) } => c.play_from_position(id - 1)?,
        SubCommand::Play { position: None } => c.play()?,
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
        ),
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
        SubCommand::ReadComments { file, plain } => {
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
                    disable_formatting: plain
                },
                width = conf.width
            );
        }
        SubCommand::Update => {
            c.update()?;
        }
        SubCommand::Status { plain } => status::status(&mut c, plain, conf.width)?,
        SubCommand::Albumart { song_path, output } => {
            albumart::fetch_albumart(&mut c, song_path.as_deref(), &*output)?;
        }
        SubCommand::Mv { from, to } => c.move_range(from - 1..=from - 1, to - 1)?,
        SubCommand::Del { index } => c.delete(index - 1..=index - 1)?,
        SubCommand::Seek { position } => seek::seek(&mut c, position)?,
        SubCommand::Custom(args) => {
            log::trace!("Spawning process for custom subcommand: {:?}", args);
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
    logger::Logger(opts.verbose).init();
    if opts.subcommand.is_none() {
        log::trace!("No subcommand specified, defaulting to current.");
        opts.subcommand = Some(SubCommand::Current {
            no_cache: false,
            plain: false,
        });
    }

    if let Some(SubCommand::Custom(v)) = &opts.subcommand {
        let mut v = v.clone();
        if let Some(subcommand) = find_subcommand(&*v[0]) {
            log::trace!("Found custom subcommand {:?}", subcommand);
            v[0] = subcommand.as_os_str().to_owned();
            opts.subcommand = Some(SubCommand::Custom(v));
        } else {
            eprintln!("{} is not a known subcommand.", v[0].to_string_lossy());
            std::process::exit(1);
        }
    }
    opts
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
    #[clap(long, short)]
    /// Enable verbose output.
    verbose: bool,
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
    Play {
        /// Queue position to start playing from, defaults to 1.
        position: Option<u32>,
    },
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
    /// Move song in queue.
    Mv {
        /// Queue index of song to move
        from: u32,
        /// Position in queue to move song to
        to: usize,
    },
    /// Remove song from queue.
    Del {
        /// Queue index
        index: u32,
    },
    /// Set current playback time.
    Seek {
        /// Position to seek to, expressed in [+-][[hh:]:mm]:ss format. If + or - is used, the seek
        /// is done relative to the current positon.
        position: seek::SeekArg,
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

fn mpd_host(opts: &Opts, conf: &config::Config) -> String {
    if let Some(host) = config::mpd_host_env_var() {
        log::trace!("Found MPD_HOST environment variable: {}", host);
        lookup_mpd_host(&*host, &conf)
    } else if let Some(host) = &opts.host {
        if let Some(host_config) = conf.hosts.iter().find(|h| h.label.as_ref() == Some(&host)) {
            log::trace!(
                "MPD host passed as label {}, and resolved to host: {}",
                host,
                host_config.host
            );
            host_config.host.clone()
        } else {
            log::trace!("Using MPD host {} from command line", host);
            host.clone()
        }
    } else {
        conf.default_mpd_host()
    }
}

fn lookup_mpd_host(host: &str, conf: &config::Config) -> String {
    if let Some(host_config) = conf.hosts.iter().find(|h| h.label.as_deref() == Some(host)) {
        log::trace!(
            "MPD host passed as label {}, and resolved to address: {}",
            host,
            host_config.host
        );
        host_config.host.clone()
    } else {
        log::trace!("MPD host is not a label, assuming address: {}", host);
        host.to_string()
    }
}
