use std::{
    error::Error as StdError,
    fmt::{self, Display},
    io::Error as IoError,
    num::ParseIntError,
};

use rcon::Error as RconError;
use serenity::Error as DiscordError;
use tracing::instrument;

/// A common error enum returned by most of the functions in this
/// application within a [`Result`].
///
/// Most error types are wrapping error types from other libraries, most
/// notably [`DiscordError`] from Serenity.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Discord(DiscordError),
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

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Discord(inner) => fmt::Display::fmt(&inner, f),
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
            Error::Io(inner) => Some(inner),
            Error::Parse(inner) => Some(inner),
            Error::Rcon(inner) => Some(inner),
            _ => None,
        }
    }
}
