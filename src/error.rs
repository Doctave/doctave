use std::fmt;
use std::path::PathBuf;

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

    pub fn broken_links(links: Vec<(PathBuf, doctave_markdown::Link)>) -> Self {
        Error {
            kind: ErrorKind::BrokenLinks(links),
            message: "Detected broken internal links".into(),
        }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    IO(std::io::Error),
    Handlebars(handlebars::RenderError),
    Yaml(serde_yaml::Error),
    BrokenLinks(Vec<(PathBuf, doctave_markdown::Link)>),
    Generic,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::IO(io_err) => write!(f, "{}:\n{}", self.message, io_err),
            ErrorKind::Handlebars(err) => write!(f, "{}:\n{}", self.message, err),
            ErrorKind::Yaml(err) => write!(f, "{}:\n{}", self.message, err),
            ErrorKind::BrokenLinks(links) => {
                write!(f, "{}\n{}", self.message, format_broken_links(&links))
            }
            ErrorKind::Generic => write!(f, "{}", self.message),
        }
    }
}

fn format_broken_links(links: &[(PathBuf, doctave_markdown::Link)]) -> String {
    let mut buf = String::from("The following links point to pages that do not exist:\n\n");

    for (path, link) in links {
        let url = match &link.url {
            doctave_markdown::UrlType::Local(path) => format!("{}", path.display()),
            doctave_markdown::UrlType::Remote(uri) => format!("{:?}", uri),
        };

        buf.push_str(&format!(
            "\t{} : [{}]({})\n",
            path.display(),
            link.title,
            url
        ));
    }

    buf
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(other: std::io::Error) -> Error {
        Error::io(other, "IO error occurred")
    }
}
