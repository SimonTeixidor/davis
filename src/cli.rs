use crate::logger;
use crate::seek;
use crate::subcommands::find_subcommand;
use std::env;
use std::ffi::OsString;

pub fn parse_args() -> Result<Opts, pico_args::Error> {
    let mut opts = pico_parse_args()?;
    logger::Logger(opts.verbose).init();
    if opts.subcommand.is_none() {
        log::trace!("No subcommand specified, defaulting to current.");
        opts.subcommand = Some(SubCommand::Current { no_cache: false });
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
    Ok(opts)
}

fn pico_parse_args() -> Result<Opts, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    if pargs.contains("--help") {
        print_help();
        std::process::exit(0);
    }

    let host = pargs.opt_value_from_str(["-h", "--host"])?;
    let verbose = pargs.contains(["-v", "--verbose"]);
    let subcommand = pargs.subcommand()?;
    let subcommand = match subcommand.as_ref().map(|s| &**s) {
        Some("current") | None => Some(SubCommand::Current {
            no_cache: pargs.contains("--no-args"),
        }),
        Some("play") => Some(SubCommand::Play {
            position: pargs.opt_free_from_str()?,
        }),
        Some("pause") => Some(SubCommand::Pause),
        Some("toggle") => Some(SubCommand::Toggle),
        Some("ls") => Some(SubCommand::Ls {
            path: pargs.opt_free_from_str()?,
        }),
        Some("clear") => Some(SubCommand::Clear),
        Some("next") => Some(SubCommand::Next),
        Some("prev") => Some(SubCommand::Prev),
        Some("stop") => Some(SubCommand::Stop),
        Some("add") => Some(SubCommand::Add {
            path: pargs.free_from_str()?,
        }),
        Some("load") => Some(SubCommand::Load {
            path: pargs.free_from_str()?,
        }),
        Some("queue") => Some(SubCommand::Queue),
        Some("search") => Some(SubCommand::Search {
            query: SearchQuery {
                query: pargs
                    .finish()
                    .into_iter()
                    .map(|os| os.into_string().unwrap())
                    .collect(),
            },
        }),
        Some("list") => Some(SubCommand::List {
            tag: pargs.free_from_str()?,
            query: SearchQuery {
                query: pargs
                    .finish()
                    .into_iter()
                    .map(|os| os.into_string().unwrap())
                    .collect(),
            },
        }),
        Some("read-comments") => Some(SubCommand::ReadComments {
            file: pargs.free_from_str()?,
        }),
        Some("update") => Some(SubCommand::Update),
        Some("status") => Some(SubCommand::Status),
        Some("albumart") => Some(SubCommand::Albumart {
            song_path: pargs.opt_free_from_str()?,
            output: pargs.value_from_str(["--output", "-o"])?,
        }),
        Some("mv") => Some(SubCommand::Mv {
            from: pargs.free_from_str()?,
            to: pargs.free_from_str()?,
        }),
        Some("del") => Some(SubCommand::Del {
            index: pargs.free_from_str()?,
        }),
        Some("seek") => Some(SubCommand::Seek {
            position: pargs.free_from_str()?,
        }),
        Some("tab") => Some(SubCommand::Tab {
            path: pargs.free_from_str()?,
        }),
        Some("help") => {
            print_help();
            std::process::exit(0);
        }
        Some(e) => {
            let mut remaining = pargs.finish();
            let cmd = e.into();
            remaining.insert(0, cmd);
            Some(SubCommand::Custom(remaining))
        }
    };

    Ok(Opts {
        host: host,
        verbose: verbose,
        subcommand: subcommand,
    })
}

pub struct Opts {
    /// The MPD server, can be specified using IP/hostname, or a label defined in the config file.
    pub host: Option<String>,
    /// Enable verbose output.
    pub verbose: bool,
    pub subcommand: Option<SubCommand>,
}

pub enum SubCommand {
    /// Display the currently playing song.
    Current {
        /// Fetch new album art from MPD, ignoring any cached images..
        no_cache: bool,
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
    Ls {
        path: Option<String>,
    },
    /// Clear the current queue.
    Clear,
    /// Skip to next song in queue.
    Next,
    /// Go back to previous song in queue.
    Prev,
    /// Stop playback.
    Stop,
    /// Add items in path to queue.
    Add {
        path: String,
    },
    /// Load playlist at path to queue.
    Load {
        path: String,
    },
    /// Display the current queue.
    Queue,
    /// Search the MPD database.
    Search {
        query: SearchQuery,
    },
    /// List all values for tag type, matching query.
    List {
        /// List values for this tag type
        tag: String,
        query: SearchQuery,
    },
    /// Read raw metadata tags for file.
    ReadComments {
        file: String,
    },
    /// Update the MPD database.
    Update,
    /// Display MPD status.
    Status,
    /// Download albumart for track.
    Albumart {
        /// The song to fetch albumart for. Will use currently playing song if not provided.
        song_path: Option<String>,
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
    Tab {
        path: String,
    },
    Custom(Vec<OsString>),
}

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

pub fn print_help() {
    println!(
        "davis {}\nSimon Persson <simon@flaskpost.me>\n",
        env!("CARGO_PKG_VERSION")
    );
    println!("{}", HELP);
}

pub static HELP: &'static str = "USAGE:
    davis [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
        --help     Prints help information
    -v, --verbose  Enable verbose output

OPTIONS:
    -h, --host <host>  IP/hostname or a label defined in the config file.

SUBCOMMANDS:
    add <path>                   Add items in path to queue.
    albumart -o <output> [path]  Download albumart.
    clear                        Clear the current queue.
    current [--no-cache]         Display the currently playing song.
    del <index>                  Remove song at index from queue.
    help                         Prints this message.
    list <tag> [query]           List all values for tag filtered by query.
    load <path>                  Load playlist at path to queue.
    ls [path]                    List items in path.
    mv <from> <to>               Move song in queue by index.
    next                         Skip to next song in queue.
    pause                        Pause playback.
    play                         Continue playback from current state.
    play [index]                 Start playback from index in queue.
    prev                         Go back to previous song in queue.
    queue                        Display the current queue.
    read-comments <file>         Read raw metadata tags for file.
    search <query>               Search for files matching query.
    seek <position>              Seek to position.
    status                       Display MPD status.
    stop                         Stop playback.
    toggle                       Toggle between play/pause.
    update                       Update the MPD database.

QUERY:
    A query can either be a single argument in the MPD filter syntax, such as:
        davis search '((artist == \"Miles Davis\") AND (album == \"Kind Of Blue\"))'
    Or a list of arguments-pairs, each pair corresponding to a filter, such as:
        davis search artist 'Miles Davis' album 'Kind Of Blue'
    More information on the MPD filter syntax is available at:
        https://mpd.readthedocs.io/en/latest/protocol.html#filters";
