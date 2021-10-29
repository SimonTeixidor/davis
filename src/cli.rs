use crate::logger;
use crate::seek;
use crate::subcommands::find_subcommand;
use clap::Clap;
use std::env;
use std::ffi::OsString;

pub fn parse_args() -> Opts {
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

#[derive(Clap)]
#[clap(bin_name = "davis", author = clap::crate_authors!(), version = clap::crate_version!())]
pub struct Opts {
    #[clap(long, short)]
    /// The MPD server, can be specified using IP/hostname, or a label defined in the config file.
    pub host: Option<String>,
    #[clap(long, short)]
    /// Enable verbose output.
    pub verbose: bool,
    #[clap(subcommand)]
    pub subcommand: Option<SubCommand>,
}

#[derive(Clap)]
pub enum SubCommand {
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
    Queue,
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
    /// List all files prefixed by path, used for tab completion.
    Tab { path: String },
    #[clap(external_subcommand)]
    Custom(Vec<OsString>),
}

#[derive(Clap)]
pub struct SearchQuery {
    /// Either a single value with a search expression, or a sequence of key-value pairs.
    pub query: Vec<String>,
}

impl SearchQuery {
    pub fn to_mpd_query(&self) -> mpd::Query {
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
