use std::fmt;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    pub fn new<S: Into<String>>(msg: S) -> Self {
        Error {
            kind: ErrorKind::Generic,
            message: msg.into(),
        }
    }

    pub fn yaml<S: Into<String>>(err: serde_yaml::Error, msg: S) -> Self {
        Error {
            kind: ErrorKind::Yaml(err),
            message: msg.into(),
        }
    }

    pub fn handlebars<S: Into<String>>(err: handlebars::RenderError, msg: S) -> Self {
        Error {
            kind: ErrorKind::Handlebars(err),
            message: msg.into(),
        }
    }

    pub fn io<S: Into<String>>(err: std::io::Error, msg: S) -> Self {
        Error {
            kind: ErrorKind::IO(err),
            message: msg.into(),
        }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    IO(std::io::Error),
    Handlebars(handlebars::RenderError),
    Yaml(serde_yaml::Error),
    Generic,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::IO(io_err) => write!(f, "{}:\n{}", self.message, io_err),
            ErrorKind::Handlebars(err) => write!(f, "{}:\n{}", self.message, err),
            ErrorKind::Yaml(err) => write!(f, "{}:\n{}", self.message, err),
            ErrorKind::Generic => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for Error {}
