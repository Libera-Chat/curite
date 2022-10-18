use std::sync::PoisonError;

pub(crate) enum Error {
    Lock,
    BadTemplate(tera::Error),
    BadArgument(&'static str),
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_error: PoisonError<T>) -> Self {
        Self::Lock
    }
}

impl From<tera::Error> for Error {
    fn from(error: tera::Error) -> Self {
        Self::BadTemplate(error)
    }
}
