use std::error::Error as StdErr;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Mpd(mpdrs::error::Error),
    Io {
        context: &'static str,
        error: std::io::Error,
    },
    ArgParse(lexopt::Error),
    ParseSeek(&'static str),
    Config(String),
}

impl StdErr for Error {}

impl From<mpdrs::error::Error> for Error {
    fn from(e: mpdrs::error::Error) -> Self {
        Error::Mpd(e)
    }
}

impl From<lexopt::Error> for Error {
    fn from(e: lexopt::Error) -> Self {
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
                write!(f, "An error occurred when communicating with MPD:\n{}", e)
            }
            Error::Io { error, context } => {
                write!(f, "I/O error when {}:\n{}", context, error)
            }
            Error::ArgParse(e) => {
                write!(f, "Failed to parse command line arguments:\n{}", e)
            }
            Error::ParseSeek(e) => {
                write!(f, "Couldn't parse seek command:\n{}", e)
            }
            Error::Config(e) => {
                write!(f, "Failed to parse config file:\n{}", e)
            }
        }
    }
}
