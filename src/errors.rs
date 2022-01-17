use std::{
    error::Error as StdError,
    fmt::{self, Display},
    io::Error as IoError,
    num::ParseIntError,
    result::Result as StdResult,
};

use rcon::Error as RconError;
use reqwest::Error as ReqwestError;
use serenity::prelude::SerenityError as DiscordError;
use tracing::instrument;

/// A common result type between many of the fuctions in this application.
pub type Result<T> = StdResult<T, Error>;

/// A common error enum returned by most of the functions in this
/// application within a [`Result`].
///
/// Most error types are wrapping error types from other libraries, most
/// notably [`DiscordError`] from Serenity.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Discord(DiscordError),
    Http(ReqwestError),
    Io(IoError),
    Other(&'static str),
    Parse(ParseIntError),
    Rcon(RconError),
}

impl From<DiscordError> for Error {
    fn from(e: DiscordError) -> Error {
        Error::Discord(e)
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Error {
        Error::Io(e)
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Error {
        Error::Parse(e)
    }
}

impl From<RconError> for Error {
    fn from(e: RconError) -> Error {
        Error::Rcon(e)
    }
}

impl From<ReqwestError> for Error {
    fn from(e: ReqwestError) -> Error {
        Error::Http(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Discord(inner) => fmt::Display::fmt(&inner, f),
            Error::Http(inner) => fmt::Display::fmt(&inner, f),
            Error::Io(inner) => fmt::Display::fmt(&inner, f),
            Error::Other(msg) => f.write_str(msg),
            Error::Parse(inner) => fmt::Display::fmt(&inner, f),
            Error::Rcon(inner) => fmt::Display::fmt(&inner, f),
        }
    }
}

impl StdError for Error {
    #[instrument]
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Discord(inner) => Some(inner),
            Error::Http(inner) => Some(inner),
            Error::Io(inner) => Some(inner),
            Error::Parse(inner) => Some(inner),
            Error::Rcon(inner) => Some(inner),
            _ => None,
        }
    }
}
