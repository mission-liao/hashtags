use std::string::String;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    GenericError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::GenericError(ref desc) => write!(f, "{}", desc),
        }
    }
}
