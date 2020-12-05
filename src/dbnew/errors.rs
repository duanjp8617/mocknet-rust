use std::fmt;
use std::convert::From;

use tokio::sync::mpsc::error as mpsc;
use tokio::sync::oneshot::error as oneshot;

/// Error kind of database backend.
#[derive(Debug, Clone, Copy)]
pub enum BackendErrorKind {
    CapnpError, // capnp rpc error, fatal
    DataError,  // data error, fatal
    InvalidArg, // input argument error, none fatal
    QueueDrop,  // the message queue has been dropped, fatal
}

/// Error of database backend.
#[derive(Debug, Clone)]
pub struct BackendError {
    kind: BackendErrorKind,
    description: String,
}

impl BackendError {
    /// Retrieve error kind from an error.
    pub fn kind(&self) -> BackendErrorKind {
        self.kind
    }

    /// Change the error message if the error kind is DataError.
    // pub fn change_data_error_msg(self, description: String) -> Self {
    //     match self.kind {
    //         BackendErrorKind::DataError => {
    //             self.description = description;
    //             self
    //         },
    //         _ => {self},
    //     }
    // }
    
    /// Create a DataError error with `description`.
    pub fn data_error(description: String) -> Self {
        Self {
            kind: BackendErrorKind::DataError,
            description,
        }
    }

    /// Create an InvalidArg error with `description`.
    pub fn invalid_arg(description: String) -> Self {
        Self {
            kind: BackendErrorKind::InvalidArg,
            description,
        }
    }
}

// Convert capnp::Error into BackendError.
impl From<capnp::Error> for BackendError {
    fn from(err: capnp::Error) -> Self {
        Self {
            kind: BackendErrorKind::CapnpError,
            description: format!("{}", err),
        }
    }
}

// Convert mpsc::SendError<T> into BackendError.
impl<T> From<mpsc::SendError<T>> for BackendError {
    fn from(_: mpsc::SendError<T>) -> BackendError {
        Self {
            kind: BackendErrorKind::QueueDrop,
            description: String::new()
        }
    }
}

// Convert oneshot::RecvError into BackendError.
impl From<oneshot::RecvError> for BackendError {
    fn from(_: oneshot::RecvError) -> BackendError {
        Self {
            kind: BackendErrorKind::QueueDrop,
            description: String::new()
        }
    }
}

// Implementing std::error::Error trait.
impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.description)         
    }
}

// Implementing std::error::Error trait.
impl std::error::Error for BackendError {}

// Necessary for converting into Box<dyn std::error::Error>.
impl From<BackendError> for Box<dyn std::error::Error + Send> {
    fn from(err: BackendError) -> Self {
        Box::new(err)
    }
}