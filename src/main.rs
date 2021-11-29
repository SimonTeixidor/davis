use mpd::lsinfo::LsInfoResponse;
use mpd::Client;
use mpd::Song;
use std::net::TcpStream;
use std::process::Command;

mod albumart;
mod ansi;
mod cli;
mod config;
mod error;
mod filecache;
mod logger;
mod now_playing;
mod queue;
mod seek;
mod status;
mod subcommands;
mod tab;
mod table;
mod tags;

use cli::SubCommand;
use error::{Error, WithContext};

fn main() {
    // Don't crash with error message on broken pipes.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    match try_main() {
        Ok(_) => (),
        Err(e @ Error::ArgParse(_)) => {
            eprintln!("{}\n\n{}", e, cli::HELP);
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

fn try_main() -> Result<(), Error> {
    let opts = cli::parse_args()?;
    let conf = config::get_config()?;

    let mpd_host = mpd_host(&opts, &conf);
    let mpd_host_str = format!("{}:6600", &mpd_host);

    let mut c = Client::new(TcpStream::connect(&mpd_host_str).context("connecting to MPD")?)?;

    match opts.subcommand.expect("no subcommand, this is a bug.") {
        SubCommand::Current { no_cache } => now_playing::now_playing(&mut c, !no_cache, &conf)?,
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
        SubCommand::Queue => queue::print(c.queue()?, c.currentsong()?),
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
        SubCommand::ReadComments { file } => {
            let table_rows = c
                .readcomments(&*trim_path(&*file))?
                .collect::<Result<Vec<_>, _>>()?;
            let table_rows = table_rows
                .iter()
                .map(|(k, v)| {
                    table::TableRow::new(vec![
                        ansi::FormattedString::new(k).style(ansi::Style::Bold),
                        ansi::FormattedString::new(v),
                    ])
                })
                .collect::<Vec<_>>();
            println!(
                "{:width$}",
                table::Table { rows: &*table_rows },
                width = conf.width
            );
        }
        SubCommand::Update => {
            c.update()?;
        }
        SubCommand::Status => status::status(&mut c)?,
        SubCommand::Albumart { song_path, output } => {
            albumart::fetch_albumart(&mut c, song_path.as_deref(), &*output)?;
        }
        SubCommand::Mv { from, to } => c.move_range(from - 1..=from - 1, to - 1)?,
        SubCommand::Del { index } => c.delete(index - 1..=index - 1)?,
        SubCommand::Seek { position } => seek::seek(&mut c, position)?,
        SubCommand::Tab { path } => tab::complete(&mut c, &*path)?,
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

fn mpd_host(opts: &cli::Opts, conf: &config::Config) -> String {
    if let Some(host) = config::mpd_host_env_var() {
        log::trace!("Found MPD_HOST environment variable: {}", host);
        lookup_mpd_host(&*host, conf)
    } else if let Some(host) = &opts.host {
        if let Some(host_config) = conf.hosts.iter().find(|h| h.label.as_ref() == Some(host)) {
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

fn trim_path(path: &str) -> &str {
    path.trim_end_matches('/')
}
