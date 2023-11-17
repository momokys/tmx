use std::error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Incomplete,
    Expect {
        message: String,
        position: usize,
        inner: Box<Error>,
    },
    Custom {
        message: String,
        position: usize,
        inner: Option<Box<Error>>,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Incomplete => write!(f, "Incomplete"),
            Self::Expect {
                ref message,
                ref position,
                ref inner,
            } => write!(f, "Expect {} at {}: {}", message, position, inner),
            Self::Custom {
                ref message,
                ref position,
                inner: Some(ref inner)
            } => write!(f, "{} at {}, {}", message, position, inner),
            Self::Custom {
                ref message,
                ref position,
                inner: None,
            } => write!(f, "{} at {}", message, position),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "Parser error"
    }
}

pub type Result<O> = std::result::Result<O, Error>;