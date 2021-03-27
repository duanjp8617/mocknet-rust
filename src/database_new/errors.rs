use std::convert::From;
use std::fmt;

use tokio::sync::mpsc::error as mpsc;
use tokio::sync::oneshot::error as oneshot;

/// Error kind of database backend.
#[derive(Debug, Clone, Copy)]
pub enum BackendErrorKind {
    QueueDrop,
}

/// Error of database backend.
#[derive(Debug)]
pub struct BackendError {
    kind: BackendErrorKind,
    description: String,
}

impl BackendError {
    /// Retrieve error kind from an error.
    pub fn kind(&self) -> BackendErrorKind {
        self.kind
    }
}

// Convert mpsc::SendError<T> into BackendError.
impl<T> From<mpsc::SendError<T>> for BackendError {
    fn from(_: mpsc::SendError<T>) -> BackendError {
        Self {
            kind: BackendErrorKind::QueueDrop,
            description: String::new(),
        }
    }
}

// Convert oneshot::RecvError into BackendError.
impl From<oneshot::RecvError> for BackendError {
    fn from(_: oneshot::RecvError) -> BackendError {
        Self {
            kind: BackendErrorKind::QueueDrop,
            description: String::new(),
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
