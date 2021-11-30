use crate::logger;
use crate::seek;
use crate::subcommands::find_subcommand;
use lexopt::prelude::*;
use std::env;
use std::ffi::OsString;
use std::num::{NonZeroU32, NonZeroUsize};

pub fn parse_args() -> Result<Opts, lexopt::Error> {
    let mut opts = lexopt_parse_args()?;
    logger::Logger(opts.verbose).init();
    if opts.subcommand.is_none() {
        log::trace!("No subcommand specified, defaulting to current.");
        opts.subcommand = Some(SubCommand::Current);
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

fn lexopt_parse_args() -> Result<Opts, lexopt::Error> {
    let mut host = None;
    let mut verbose = false;
    let mut plain_formatting = false;

    let mut parser = lexopt::Parser::from_env();
    let mut subcommand = None;
    while let Some(arg) = parser.next()? {
        match arg {
            Long("help") => {
                print_help();
                std::process::exit(0);
            }
            Value(s) if s.clone().into_string()? == "help" => {
                print_help();
                std::process::exit(0);
            }
            Short('h') => {
                host = Some(parser.value()?.parse()?);
            }
            Short('v') | Long("verbose") => {
                verbose = true;
            }
            Short('p') | Long("plain") => {
                plain_formatting = true;
            }
            Value(cmd) => {
                let cmd = cmd.into_string()?;
                subcommand = Some(match &*cmd {
                    "current" => SubCommand::Current,
                    "play" => SubCommand::Play {
                        position: if let Some(Value(i)) = parser.next()? {
                            Some(i.parse()?)
                        } else {
                            None
                        },
                    },
                    "pause" => SubCommand::Pause,
                    "toggle" => SubCommand::Toggle,
                    "ls" => SubCommand::Ls {
                        path: if let Some(Value(i)) = parser.next()? {
                            Some(i.parse()?)
                        } else {
                            None
                        },
                    },
                    "clear" => SubCommand::Clear,
                    "next" => SubCommand::Next,
                    "prev" => SubCommand::Prev,
                    "stop" => SubCommand::Stop,
                    "add" => SubCommand::Add {
                        path: if let Some(Value(i)) = parser.next()? {
                            i.parse()?
                        } else {
                            return Err("missing path argument".into());
                        },
                    },
                    "load" => SubCommand::Load {
                        path: if let Some(Value(i)) = parser.next()? {
                            i.parse()?
                        } else {
                            return Err("missing path argument".into());
                        },
                    },
                    "queue" => SubCommand::Queue,
                    "search" => {
                        let mut query = vec![];
                        while let Some(Value(i)) = parser.next()? {
                            query.push(i.into_string()?);
                        }
                        SubCommand::Search {
                            query: SearchQuery { query },
                        }
                    }
                    "list" => {
                        let tag = if let Some(Value(i)) = parser.next()? {
                            i.into_string()?
                        } else {
                            return Err("missing tag argument".into());
                        };

                        let mut query = vec![];
                        while let Some(Value(i)) = parser.next()? {
                            query.push(i.into_string()?);
                        }

                        SubCommand::List {
                            tag,
                            query: SearchQuery { query },
                        }
                    }
                    "read-comments" => SubCommand::ReadComments {
                        file: if let Some(Value(i)) = parser.next()? {
                            i.parse()?
                        } else {
                            return Err("missing file argument".into());
                        },
                    },
                    "update" => SubCommand::Update,
                    "status" => SubCommand::Status,
                    "albumart" => {
                        let mut output = None;
                        let mut song_path = None;
                        while let Some(arg) = parser.next()? {
                            match arg {
                                Short('o') | Long("output") => {
                                    output = Some(parser.value()?.parse()?)
                                }
                                Value(path) => song_path = Some(path.into_string()?),
                                _ => return Err(arg.unexpected()),
                            }
                        }
                        SubCommand::Albumart {
                            output: output.ok_or("missing output option")?,
                            song_path,
                        }
                    }
                    "mv" => SubCommand::Mv {
                        from: if let Some(Value(i)) = parser.next()? {
                            i.parse()?
                        } else {
                            return Err("missing from argument".into());
                        },
                        to: if let Some(Value(i)) = parser.next()? {
                            i.parse()?
                        } else {
                            return Err("missing to argument".into());
                        },
                    },
                    "del" => SubCommand::Del {
                        index: if let Some(Value(i)) = parser.next()? {
                            i.parse()?
                        } else {
                            return Err("missing index argument".into());
                        },
                    },
                    "seek" => SubCommand::Seek {
                        position: if let Some(Value(i)) = parser.next()? {
                            i.parse()?
                        } else {
                            return Err("missing position argument".into());
                        },
                    },
                    "tab" => SubCommand::Tab {
                        path: if let Some(Value(i)) = parser.next()? {
                            i.parse()?
                        } else {
                            return Err("missing path argument".into());
                        },
                    },
                    cmd => {
                        let mut remaining = vec![];
                        while let Some(Value(i)) = parser.next()? {
                            remaining.push(i);
                        }
                        let cmd = cmd.into();
                        remaining.insert(0, cmd);
                        SubCommand::Custom(remaining)
                    }
                })
            }
            _ => return Err(arg.unexpected()),
        }
    }

    Ok(Opts {
        host,
        verbose,
        plain_formatting,
        subcommand,
    })
}

pub struct Opts {
    pub host: Option<String>,
    pub verbose: bool,
    pub plain_formatting: bool,
    pub subcommand: Option<SubCommand>,
}

pub enum SubCommand {
    Current,
    Play {
        position: Option<NonZeroU32>,
    },
    Pause,
    Toggle,
    Ls {
        path: Option<String>,
    },
    Clear,
    Next,
    Prev,
    Stop,
    Add {
        path: String,
    },
    Load {
        path: String,
    },
    Queue,
    Search {
        query: SearchQuery,
    },
    List {
        tag: String,
        query: SearchQuery,
    },
    ReadComments {
        file: String,
    },
    Update,
    Status,
    Albumart {
        song_path: Option<String>,
        output: String,
    },
    Mv {
        from: NonZeroU32,
        to: NonZeroUsize,
    },
    Del {
        index: NonZeroU32,
    },
    Seek {
        position: seek::SeekArg,
    },
    Tab {
        path: String,
    },
    Custom(Vec<OsString>),
}

pub struct SearchQuery {
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

pub static HELP: &str = "USAGE:
    davis [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
        --help     Prints help information.
    -v, --verbose  Enable verbose output.
    -p, --plain    Disable decorations in output, useful for scripting.

OPTIONS:
    -h, --host <host>  IP/hostname or a label defined in the config file.

SUBCOMMANDS:
    add <path>                   Add items in path to queue.
    albumart -o <output> [path]  Download albumart.
    clear                        Clear the current queue.
    current                      Display the currently playing song.
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
