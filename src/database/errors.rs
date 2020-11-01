use std::fmt;
use std::convert::From;

use tokio::sync::mpsc::error as mpsc;
use tokio::sync::oneshot::error as oneshot;

// Error of indradb backend
#[derive(Debug, Clone, Copy)]
pub enum BackendErrorKind {
    CapnpError,
    InvalidArg,
}

#[derive(Debug, Clone)]
pub struct BackendError {
    kind: BackendErrorKind,
    description: String,
}

impl BackendError {
    pub fn kind(&self) -> BackendErrorKind {
        self.kind
    }

    pub fn invalid_arg(description: String) -> Self {
        Self {
            kind: BackendErrorKind::InvalidArg,
            description,
        }
    }
}

impl From<capnp::Error> for BackendError {
    fn from(err: capnp::Error) -> Self {
        Self {
            kind: BackendErrorKind::CapnpError,
            description: format!("{}", err),
        }
    }
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.description)         
    }
}

impl std::error::Error for BackendError {}

// Error of message queue
#[derive(Debug)]
pub enum MsgQError<E: std::error::Error> {
    QueueDrop,
    Inner(E),
}

// If we don't enforce E to implement StdError, then rust complains.
impl<E: std::error::Error, T> From<mpsc::SendError<T>> for MsgQError<E> {
    fn from(_: mpsc::SendError<T>) -> MsgQError<E> {
        MsgQError::<E>::QueueDrop
    }
}

impl<E: std::error::Error> From<oneshot::RecvError> for MsgQError<E> {
    fn from(_: oneshot::RecvError) -> MsgQError<E> {
        MsgQError::QueueDrop
    }
}

impl<E: std::error::Error> fmt::Display for MsgQError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MsgQError::QueueDrop => {
                write!(f, "message queue is dropped")
            }
            MsgQError::Inner(e) => {
                write!(f, "inner error message: {}", e)
            }
        }            
    }
}

impl<E: std::error::Error> std::error::Error for MsgQError<E> {}

// If we don't enforce E to implement StdError, then rust complains.
// Because MsgQError::Inner is only available for type variable E 
// that implement std::error::Error
impl<E: std::error::Error> MsgQError<E> {
    pub fn get_inner(self) -> Option<E> {
        match self {
            MsgQError::Inner(e) => Some(e),
            _ => None
        }
    }

    pub(super) fn from_error(e: E) -> Self {
        MsgQError::Inner(e)
    }
}