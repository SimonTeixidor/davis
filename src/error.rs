use std::fmt;

#[derive(Debug)]
pub enum Error {
    MpdError(mpd::error::Error),
    IoError {
        context: &'static str,
        error: std::io::Error,
    },
    PicoError(pico_args::Error),
    ImageError(image::ImageError),
    LiqError(imagequant::liq_error),
    ConfigError(tini::Error),
}

impl From<image::ImageError> for Error {
    fn from(e: image::ImageError) -> Self {
        Error::ImageError(e)
    }
}

impl From<imagequant::liq_error> for Error {
    fn from(e: imagequant::liq_error) -> Self {
        Error::LiqError(e)
    }
}

impl From<tini::Error> for Error {
    fn from(e: tini::Error) -> Self {
        Error::ConfigError(e)
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

impl From<pico_args::Error> for Error {
    fn from(e: pico_args::Error) -> Self {
        Error::PicoError(e)
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
                write!(f, "MPD Error: {}", e)
            }
            Error::IoError { error, context } => {
                write!(f, "IoError: {}, context: {}", error, context)
            }
            Error::PicoError(e) => {
                write!(f, "Argument parsing error: {}:", e)
            }
            Error::ImageError(e) => {
                write!(f, "Image Error: {}:", e)
            }
            Error::LiqError(e) => {
                write!(f, "Image Error: {}:", e)
            }
            Error::ConfigError(e) => {
                write!(f, "Config Error: {}:", e)
            }
        }
    }
}
