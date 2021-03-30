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
}

impl From<image::ImageError> for Error {
    fn from(e: image::ImageError) -> Self {
        Error::ImageError(e)
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
        }
    }
}
