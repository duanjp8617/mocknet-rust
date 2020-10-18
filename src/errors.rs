use std::{convert::From, fmt, error::Error as StdError};

use capnp::Error as CapnpError;
use std::io::Error as StdIoError;

pub struct Error {
    pub kind: Kind,
    pub description: String,
}

#[derive(Debug)]
pub enum Kind {
    // Self-defined error types:
    Wtf,
    
    // Error types generated from dependencies
    CapnpError,
    StdIoError,
}

// Build self-defined error types
impl Error {
    pub fn wtf(description: String) -> Self {
        Self {
            kind: Kind::Wtf,
            description,
        }
    }

    pub fn capnp_error(description: String) -> Self {
        Self {
            kind: Kind::CapnpError,
            description,
        }
    }
}
 
// Convert errors from dependencies to errors::Error
impl From<CapnpError> for Error {
    fn from(err: CapnpError) -> Self {
        Self {
            kind: Kind::CapnpError,
            description: format!("{}", err)
        }
    }
}

impl From<StdIoError> for Error {
    fn from(err: StdIoError) -> Self {
        Self {
            kind: Kind::StdIoError,
            description: format!("{}", err)
        }
    }
}

// Implement necessary traits to make errors::Error an error
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_tuple("mocknet::errors::Error");
        f.field(&self.kind);
        f.field(&self.description);
        f.finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", &self.kind, &self.description)
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        &self.description
    }
}

