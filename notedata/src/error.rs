use std::{error, fmt, io};

type NomError<T = String> = nom::Err<(T, nom::error::ErrorKind)>;

#[derive(Debug)]
pub enum ParseError {
    Io(io::Error),
    Parse(NomError),
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Io(error) => error.source(),
            Self::Parse(error) => error.source(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(f),
            Self::Parse(error) => error.fmt(f),
        }
    }
}

impl From<io::Error> for ParseError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<NomError<&str>> for ParseError {
    fn from(error: NomError<&str>) -> Self {
        Self::Parse(error.to_owned())
    }
}
