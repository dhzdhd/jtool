use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    JSONParsing(serde_json::Error),
    JSONStringify(String),
    NotImplemented,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::JSONParsing(err) => write!(f, "Error while parsing JSON {}", err),
            Error::JSONStringify(msg) => write!(f, "Error while stringifying JSON: {}", msg),
            Error::NotImplemented => write!(f, "Not implemented yet"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::JSONParsing(err) => Some(err),
            Error::JSONStringify(_) | Error::NotImplemented => None,
        }
    }
}
