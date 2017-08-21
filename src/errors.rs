use std;
use std::error;
use std::fmt;
use std::io;

#[allow(dead_code)]
/// Re-exported `Result` for rcue errors
pub type Result<T> = std::result::Result<T, CueError>;

#[allow(dead_code)]
#[derive(Debug)]
/// Represents a parsing error.
pub enum CueError {
    /// CUE parse error
    Parse(String),
    /// IO error (file could not read)
    Io(io::Error),
}

impl fmt::Display for CueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CueError::Parse(ref token) => write!(f, "Parse error: {}", token),
            CueError::Io(ref err) => write!(f, "Io error: {}", err),
        }
    }
}

impl error::Error for CueError {
    fn description(&self) -> &str {
        match *self {
            CueError::Parse(ref token) => token,
            CueError::Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            CueError::Parse(ref _token) => None,
            CueError::Io(ref err) => err.cause(),
        }
    }
}

impl From<io::Error> for CueError {
    fn from(err: io::Error) -> Self {
        CueError::Io(err)
    }
}

impl From<std::num::ParseFloatError> for CueError {
    fn from(err: std::num::ParseFloatError) -> Self {
        CueError::Parse(format!("invalid timestamp: {}", err))
    }
}

impl From<std::num::ParseIntError> for CueError {
    fn from(err: std::num::ParseIntError) -> Self {
        CueError::Parse(format!("invalid timestamp: {}", err))
    }
}
