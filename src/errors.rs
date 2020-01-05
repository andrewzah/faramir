use std::{fmt, result};

pub type AppResult<T> = result::Result<T, AppError>;

pub struct AppError(Box<ErrorKind>);

impl AppError {
    pub(crate) fn new(kind: ErrorKind) -> AppError {
        AppError(Box::new(kind))
    }

    pub(crate) fn from_str(msg: &str) -> AppError {
        AppError(Box::new(ErrorKind::Generic(msg.into())))
    }

    pub(crate) fn kind(&self) -> &ErrorKind {
        &self.0
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    Generic(String),
    IO(std::io::Error),
    SerdeJson(serde_json::error::Error),
    Rusqlite(rusqlite::Error),
    ChronoParse(chrono::format::ParseError),
    StringParse(std::string::String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self.0 {
            ErrorKind::Generic(ref msg) => write!(f, "{}", msg),
            ErrorKind::IO(ref err) => err.fmt(f),
            ErrorKind::SerdeJson(ref err) => err.fmt(f),
            ErrorKind::Rusqlite(ref err) => err.fmt(f),
            ErrorKind::ChronoParse(ref err) => err.fmt(f),
            ErrorKind::StringParse(ref err) => err.fmt(f),
        }
    }
}

impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self.0 {
            ErrorKind::Generic(ref msg) => write!(f, "Faramir Error: {}", msg),
            ErrorKind::IO(ref err) => err.fmt(f),
            ErrorKind::SerdeJson(ref err) => err.fmt(f),
            ErrorKind::Rusqlite(ref err) => err.fmt(f),
            ErrorKind::ChronoParse(ref err) => err.fmt(f),
            ErrorKind::StringParse(ref err) => err.fmt(f),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> AppError {
        AppError::new(ErrorKind::IO(err))
    }
}

impl From<serde_json::error::Error> for AppError {
    fn from(err: serde_json::error::Error) -> AppError {
        AppError::new(ErrorKind::SerdeJson(err))
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> AppError {
        AppError::new(ErrorKind::Rusqlite(err))
    }
}

impl From<chrono::format::ParseError> for AppError {
    fn from(err: chrono::format::ParseError) -> AppError {
        AppError::new(ErrorKind::ChronoParse(err))
    }
}

impl From<std::string::String> for AppError {
    fn from(err: std::string::String) -> AppError {
        AppError::new(ErrorKind::StringParse(err))
    }
}
