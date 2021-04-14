use std::fmt;

#[derive(Debug)]
pub enum Error {
    MpdError(mpd::error::Error),
    IoError {
        context: &'static str,
        error: std::io::Error,
    },
    ImageError(image::ImageError),
    LiqError(imagequant::liq_error),
    TomlError(toml::de::Error),
}

impl From<image::ImageError> for Error {
    fn from(e: image::ImageError) -> Self {
        Error::ImageError(e)
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Error::TomlError(e)
    }
}

impl From<imagequant::liq_error> for Error {
    fn from(e: imagequant::liq_error) -> Self {
        Error::LiqError(e)
    }
}

impl From<sixel::Error> for Error {
    fn from(e: sixel::Error) -> Self {
        match e {
            sixel::Error::LiqError(e) => Error::LiqError(e),
            sixel::Error::IoError(error) => Error::IoError {
                context: "writing sixel image",
                error,
            },
        }
    }
}

impl From<mpd::error::Error> for Error {
    fn from(e: mpd::error::Error) -> Self {
        Error::MpdError(e)
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
            Error::ImageError(e) => {
                write!(f, "An error occured when processing the album art: {}", e)
            }
            Error::LiqError(e) => {
                write!(f, "An error occured when processing the album art: {}", e)
            }
            Error::TomlError(e) => {
                write!(f, "Couldn't parse the configuration file:\n{}", e)
            }
        }
    }
}
