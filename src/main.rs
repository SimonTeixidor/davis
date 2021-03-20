use mpd::Client;
use std::net::TcpStream;

mod ansi;
mod error;
mod now_playing;
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
            println!("{}", e);
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
    match subcommand {
        SubCommand::NowPlaying => {
            let winsize = terminal_dimensions::terminal_size();
            let mut c = Client::new(
                TcpStream::connect("127.0.0.1:6600").context("Failed to connect to MPD.")?,
            )?;
            now_playing::now_playing(&mut c, &winsize).unwrap();
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

    let subcommand = match pargs.subcommand()?.as_ref().map(|s| &**s) {
        Some("current") => SubCommand::NowPlaying,
        None => SubCommand::NowPlaying,
        Some(s) => {
            return Err(pico_args::Error::ArgumentParsingFailed {
                cause: format!("unknown subcommand {}", s),
            })
        }
    };
    Ok(SubCommand::NowPlaying)
}

enum SubCommand {
    NowPlaying,
}

static HELP: &str = "\
USAGE:
  davis [current]     Display currently playing song
";
