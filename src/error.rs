use std::error::Error as StdErr;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    MpdError(mpd::error::Error),
    IoError {
        context: &'static str,
        error: std::io::Error,
    },
    TomlError(toml::de::Error),
    ParseSeekError(&'static str),
}

impl StdErr for Error {}

impl From<mpd::error::Error> for Error {
    fn from(e: mpd::error::Error) -> Self {
        Error::MpdError(e)
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Error::TomlError(e)
    }
}

pub trait WithContext<T> {
    fn context(self, ctx: &'static str) -> Result<T, Error>;
}

impl<T> WithContext<T> for Result<T, std::io::Error> {
    fn context(self, context: &'static str) -> Result<T, Error> {
        self.map_err(|error| Error::IoError { context, error })
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MpdError(e) => {
                write!(f, "An error occured when communicating with MPD: {}", e)
            }
            Error::IoError { error, context } => {
                write!(f, "I/O error when {}:\n{}", context, error)
            }
            Error::TomlError(e) => {
                write!(f, "Couldn't parse the configuration file:\n{}", e)
            }
            Error::ParseSeekError(e) => {
                write!(f, "Couldn't parse seek command: {}", e)
            }
        }
    }
}
