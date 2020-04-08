use std::convert::From;
use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    Utf8Error(std::str::Utf8Error),
    MalformedVorbisComment(String),
    InvalidFlacHeader(PathBuf),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Error::IoError(e) => write!(f, "I/O Error: {}", e),
            Error::Utf8Error(e) => write!(f, "UTF8 error: {}", e),
            Error::MalformedVorbisComment(e) => write!(f, "Malformed vorbis comment: {}", e),
            Error::InvalidFlacHeader(p) => write!(f, "Invalid flac file: {}", p.display()),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Error {
        Error::Utf8Error(e)
    }
}
