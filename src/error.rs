use tera;
use toml;
use serde_json;

use std::io;
use std::result;


#[derive(Debug)]
pub enum TechouError {
    IO { source: io::Error, context: String },

    FrontMatter { issue: String },

    Templating {
        source: tera::Error,
        context: String,
    },

    TOML {
        source: toml::de::Error,
        context: String,
    },

    Other { issue: String },

    JSON {
        source: serde_json::error::Error,
        context: String,
    },
}

impl std::fmt::Display for TechouError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Some sort of error: {:?}", &self)
    }
}


pub type Result<T> = result::Result<T, TechouError>;

pub trait ResultContext<T> {
    fn ctx<A: std::fmt::Debug>(self, ctx: A) -> Result<T>;
}

impl<T> ResultContext<T> for result::Result<T, io::Error> {
    fn ctx<A: std::fmt::Debug>(self, ctx: A) -> Result<T> {
        self.map_err(|e| TechouError::IO {
            source: e,
            context: format!("{:?}", ctx),
        })
    }
}

impl<T> ResultContext<T> for result::Result<T, tera::Error> {
    fn ctx<A: std::fmt::Debug>(self, ctx: A) -> Result<T> {
        self.map_err(|e| TechouError::Templating {
            source: e,
            context: format!("{:?}", ctx),
        })
    }
}

impl<T> ResultContext<T> for result::Result<T, toml::de::Error> {
    fn ctx<A: std::fmt::Debug>(self, ctx: A) -> Result<T> {
        self.map_err(|e| TechouError::TOML {
            source: e,
            context: format!("{:?}", ctx),
        })
    }
}

impl<T> ResultContext<T> for result::Result<T, serde_json::error::Error> {
    fn ctx<A: std::fmt::Debug>(self, ctx: A) -> Result<T> {
        self.map_err(|e| TechouError::JSON {
            source: e,
            context: format!("{:?}", ctx),
        })
    }
}
