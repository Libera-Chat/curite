use std::sync::PoisonError;

pub(crate) enum Error {
    Lock,
    BadTemplate(askama::Error),
    BadTemplate2(tinytemplate::error::Error),
    BadArgument(&'static str),
}

impl From<askama::Error> for Error {
    fn from(error: askama::Error) -> Self {
        Self::BadTemplate(error)
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_error: PoisonError<T>) -> Self {
        Self::Lock
    }
}

impl From<tinytemplate::error::Error> for Error {
    fn from(error: tinytemplate::error::Error) -> Self {
        Self::BadTemplate2(error)
    }
}
