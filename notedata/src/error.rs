#[cfg(feature = "std")]
use crate::lib::io;
use crate::lib::{fmt, Error, String};

use nom;

#[derive(Debug)]
pub enum ParseError {
    #[cfg(feature = "std")]
    Io(io::Error),
    Parse(nom::Err<(String, nom::error::ErrorKind)>),
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            #[cfg(feature = "std")]
            Self::Io(error) => error.source(),
            Self::Parse(_) => None,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            #[cfg(feature = "std")]
            Self::Io(error) => error.fmt(f),
            Self::Parse(error) => error.fmt(f),
        }
    }
}

#[cfg(feature = "std")]
impl From<io::Error> for ParseError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<nom::Err<(&str, nom::error::ErrorKind)>> for ParseError {
    fn from(error: nom::Err<(&str, nom::error::ErrorKind)>) -> Self {
        Self::Parse(error.to_owned())
    }
}
