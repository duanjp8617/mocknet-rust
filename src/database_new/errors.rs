use std::convert::From;
use std::fmt;

use http::uri::InvalidUri;
use indradb_proto::ClientError;

use super::message_queue::errors::QueueDrop;

/// Error kind of database backend.
#[derive(Debug)]
pub enum ConnError {
    IndradbClientError { inner: ClientError },
    AddressError { inner: InvalidUri },
    QueueDrop,
}

// Convert mpsc::SendError<T> into BackendError.
impl From<QueueDrop> for ConnError {
    fn from(_: QueueDrop) -> ConnError {
        Self::QueueDrop
    }
}

// Convert oneshot::RecvError into BackendError.
impl From<ClientError> for ConnError {
    fn from(e: ClientError) -> ConnError {
        Self::IndradbClientError { inner: e }
    }
}

impl From<InvalidUri> for ConnError {
    fn from(e: InvalidUri) -> ConnError {
        Self::AddressError { inner: e }
    }
}

// Implementing std::error::Error trait.
impl fmt::Display for ConnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnError::IndradbClientError { ref inner } => {
                write!(f, "Indradb client error: {}", inner)
            }
            ConnError::AddressError { ref inner } => {
                write!(f, "address error: {}", inner)
            }
            ConnError::QueueDrop => {
                write!(f, "queue drop error")
            }
        }
    }
}

// Implementing std::error::Error trait.
impl std::error::Error for ConnError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            ConnError::IndradbClientError { ref inner } => Some(inner),
            ConnError::AddressError { ref inner } => Some(inner),
            _ => None,
        }
    }
}

// Necessary for converting into Box<dyn std::error::Error>.
impl From<ConnError> for Box<dyn std::error::Error + Send + 'static> {
    fn from(err: ConnError) -> Self {
        Box::new(err)
    }
}
