use std::error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::result;

use tera;
use toml;

use err_derive::*;

#[derive(Debug, err_derive::Error)]
pub enum TechouError {

    #[error(display = "io error with {}: {}", context, source)]
    IO { source: io::Error, context: String },

    #[error(display = "invalid front-matter: {:?}", issue)]
    FrontMatter { issue: String },

    #[error(display = "templating error with {}: {}", context, source)]
    Templating { source: tera::Error, context: String },

    #[error(display = "toml error with {}: {}", context, source)]
    TOML { source: toml::de::Error, context: String },

    #[error(display = "other: {:?}", issue)]
    Other { issue: String },
}

pub type Result<T> = result::Result<T, TechouError>;

pub trait ResultContext<T> {
    fn ctx<A: std::fmt::Debug>(self, ctx: A) -> Result<T>;
}

impl<T> ResultContext<T> for result::Result<T, io::Error> {
    fn ctx<A: std::fmt::Debug>(self, ctx: A) -> Result<T> {
        self.map_err(|e| TechouError::IO {
            source: e,
            context: format!("{:?}", ctx)
        })
    }
}

impl<T> ResultContext<T> for result::Result<T, tera::Error> {
    fn ctx<A: std::fmt::Debug>(self, ctx: A) -> Result<T> {
        self.map_err(|e| TechouError::Templating {
            source: e,
            context: format!("{:?}", ctx)
        })
    }
}

impl<T> ResultContext<T> for result::Result<T, toml::de::Error> {
    fn ctx<A: std::fmt::Debug>(self, ctx: A) -> Result<T> {
        self.map_err(|e| TechouError::TOML {
            source: e,
            context: format!("{:?}", ctx)
        })
    }
}
