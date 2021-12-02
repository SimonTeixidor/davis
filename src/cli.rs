use crate::logger;
use crate::seek;
use crate::subcommands::find_subcommand;
use lexopt::prelude::*;
use std::env;
use std::ffi::OsString;
use std::num::{NonZeroU32, NonZeroUsize};
use std::str::FromStr;

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

fn next_arg<T: FromStr>(name: &str, parser: &mut lexopt::Parser) -> Result<T, lexopt::Error>
where
    T::Err: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
{
    if let Some(Value(i)) = parser.next()? {
        i.parse()
    } else {
        Err(format!("missing argument: {}", name).into())
    }
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
                subcommand = Some(parse_subcommand(cmd, &mut parser)?);
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

fn parse_subcommand(
    cmd: OsString,
    parser: &mut lexopt::Parser,
) -> Result<SubCommand, lexopt::Error> {
    let cmd = cmd.into_string()?;
    Ok(match &*cmd {
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
            path: next_arg("path", parser)?,
        },
        "load" => SubCommand::Load {
            path: next_arg("path", parser)?,
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
            let tag = next_arg("tag", parser)?;
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
            file: next_arg("file", parser)?,
        },
        "update" => SubCommand::Update,
        "status" => SubCommand::Status,
        "albumart" => {
            let mut output = None;
            let mut song_path = None;
            while let Some(arg) = parser.next()? {
                match arg {
                    Short('o') | Long("output") => {
                        output = Some(parser.value()?.parse()?);
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
            from: next_arg("from", parser)?,
            to: next_arg("to", parser)?,
        },
        "del" => SubCommand::Del {
            index: next_arg("index", parser)?,
        },
        "seek" => SubCommand::Seek {
            position: next_arg("position", parser)?,
        },
        "tab" => SubCommand::Tab {
            path: next_arg("path", parser).unwrap_or_else(|_| "".into()),
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
        position: seek::Arg,
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
    davis add <path>                   Add items in path to queue.
    davis albumart -o <output> [path]  Download albumart.
    davis clear                        Clear the current queue.
    davis current                      Display the currently playing song.
    davis del <index>                  Remove song at index from queue.
    davis help                         Prints this message.
    davis list <tag> [query]           List values for tag filtered by query.
    davis load <path>                  Load playlist at path to queue.
    davis ls [path]                    List items in path.
    davis mv <from> <to>               Move song in queue by index.
    davis next                         Skip to next song in queue.
    davis pause                        Pause playback.
    davis play                         Continue playback from current state.
    davis play [index]                 Start playback from index in queue.
    davis prev                         Go back to previous song in queue.
    davis queue                        Display the current queue.
    davis read-comments <file>         Read raw metadata tags for file.
    davis search <query>               Search for files matching query.
    davis seek <position>              Seek to position.
    davis status                       Display MPD status.
    davis stop                         Stop playback.
    davis toggle                       Toggle between play/pause.
    davis update                       Update the MPD database.

QUERY:
    A query can either be a single argument in the MPD filter syntax, such as:
        davis search '((artist == \"Miles Davis\") AND (album == \"Kind Of Blue\"))'
    Or a list of arguments-pairs, each pair corresponding to a filter, such as:
        davis search artist 'Miles Davis' album 'Kind Of Blue'
    More information on the MPD filter syntax is available at:
        https://mpd.readthedocs.io/en/latest/protocol.html#filters";
