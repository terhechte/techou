use std::error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::result;

use tera;
use fs_extra;
use toml;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    FrontMatter(String),
    Templating(tera::Error),
    Other(String)
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref error) => error.fmt(formatter),
            Error::FrontMatter(ref error) => error.fmt(formatter),
            Error::Templating(ref error) => error.fmt(formatter),
            Error::Other(ref error) => error.fmt(formatter),
        }
    }
}

impl error::Error for Error {
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<tera::Error> for Error {
    fn from(error: tera::Error) -> Self { Error::Templating(error) }
}

impl From<fs_extra::error::Error> for Error {
    fn from(error: fs_extra::error::Error) -> Self { Error::Other(format!("Copy Error: {:?}", &error)) }
}

impl From<toml::de::Error> for Error {
    fn from(error: toml::de::Error) -> Self { Error::Other(format!("Could not parse toml: {:?}", &error)) }
}

pub type Result<T> = result::Result<T, Error>;