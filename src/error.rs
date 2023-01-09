use std::sync::PoisonError;

#[derive(Debug)]
pub enum Error {
    Lock,
    BadTemplate(tera::Error),
    BadArgument(&'static str),
    Std(std::io::Error),
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

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Std(error)
    }
}
