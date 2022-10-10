pub(crate) enum Error {
    BadTemplate(askama::Error),
    BadArgument(&'static str),
}

impl From<askama::Error> for Error {
    fn from(error: askama::Error) -> Self {
        Self::BadTemplate(error)
    }
}
