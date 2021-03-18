#[derive(Debug)]
pub enum Error {
    SixelError(sixel::status::Error),
    MpdError(mpd::error::Error),
    IoError {
        context: &'static str,
        error: std::io::Error,
    },
}

impl From<sixel::status::Error> for Error {
    fn from(e: sixel::status::Error) -> Self {
        Error::SixelError(e)
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
