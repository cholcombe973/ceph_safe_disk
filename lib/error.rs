use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::string;

use serde_json::error;

#[derive(Debug)]
pub enum CSDError {
    Io(io::Error),
    JsonDecode(error::Error),
    Utf8Error(string::FromUtf8Error),
    CephExecError(String),
    ExecError,
}

impl fmt::Display for CSDError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CSDError::Io(ref err) => write!(f, "I/O error, {}", err),
            CSDError::JsonDecode(ref err) => write!(f, "JSON decoding error, {}", err),
            CSDError::Utf8Error(ref err) => write!(f, "UTF-8 conversion error, {}", err),
            CSDError::CephExecError(ref err) => write!(f, "Error executing `ceph`, {}", err),
            CSDError::ExecError => write!(f, "Must be run as root or ceph user"),
        }
    }
}

impl StdError for CSDError {
    fn description(&self) -> &str {
        match *self {
            CSDError::Io(ref err) => err.description(),
            CSDError::JsonDecode(ref err) => err.description(),
            CSDError::Utf8Error(ref err) => err.description(),
            CSDError::CephExecError(ref err) => err,
            CSDError::ExecError => "Must be run as root or ceph user",
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match *self {
            CSDError::Io(ref err) => err.cause(),
            CSDError::JsonDecode(ref err) => err.cause(),
            CSDError::Utf8Error(ref err) => err.cause(),
            CSDError::CephExecError(ref _err) => None,
            CSDError::ExecError => None,
        }
    }
}

impl From<io::Error> for CSDError {
    fn from(err: io::Error) -> CSDError {
        CSDError::Io(err)
    }
}

impl From<error::Error> for CSDError {
    fn from(err: error::Error) -> CSDError {
        CSDError::JsonDecode(err)
    }
}

impl From<string::FromUtf8Error> for CSDError {
    fn from(err: string::FromUtf8Error) -> CSDError {
        CSDError::Utf8Error(err)
    }
}
