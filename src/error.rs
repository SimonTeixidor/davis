use std::error::Error as StdErr;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Mpd(mpd::error::Error),
    Io {
        context: &'static str,
        error: std::io::Error,
    },
    ArgParse(pico_args::Error),
    Toml(toml::de::Error),
    ParseSeek(&'static str),
}

impl StdErr for Error {}

impl From<mpd::error::Error> for Error {
    fn from(e: mpd::error::Error) -> Self {
        Error::Mpd(e)
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Error::Toml(e)
    }
}

impl From<pico_args::Error> for Error {
    fn from(e: pico_args::Error) -> Self {
        Error::ArgParse(e)
    }
}

pub trait WithContext<T> {
    fn context(self, ctx: &'static str) -> Result<T, Error>;
}

impl<T> WithContext<T> for Result<T, std::io::Error> {
    fn context(self, context: &'static str) -> Result<T, Error> {
        self.map_err(|error| Error::Io { context, error })
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Mpd(e) => {
                write!(f, "An error occured when communicating with MPD: {}", e)
            }
            Error::Io { error, context } => {
                write!(f, "I/O error when {}:\n{}", context, error)
            }
            Error::Toml(e) => {
                write!(f, "Couldn't parse the configuration file:\n{}", e)
            }
            Error::ArgParse(e) => {
                write!(f, "Failed to parse command line arguments:\n{}", e)
            }
            Error::ParseSeek(e) => {
                write!(f, "Couldn't parse seek command: {}", e)
            }
        }
    }
}
