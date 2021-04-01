use std::convert::From;
use std::fmt;

use http::uri::InvalidUri;
use indradb_proto::ClientError;
use tokio::time::error::Elapsed;

use super::message_queue::errors::QueueDrop;

#[derive(Debug)]
pub enum ConnectorError {
    ConnectionError { reason: String },
    QueueDrop,
}

impl From<QueueDrop> for ConnectorError {
    fn from(_: QueueDrop) -> ConnectorError {
        Self::QueueDrop
    }
}

impl From<ClientError> for ConnectorError {
    fn from(e: ClientError) -> ConnectorError {
        Self::ConnectionError {
            reason: format!("{}", e),
        }
    }
}

impl From<InvalidUri> for ConnectorError {
    fn from(e: InvalidUri) -> ConnectorError {
        Self::ConnectionError {
            reason: format!("{}", e),
        }
    }
}

impl From<Elapsed> for ConnectorError {
    fn from(e: Elapsed) -> ConnectorError {
        Self::ConnectionError {
            reason: format!("connection timeout: {}", e),
        }
    }
}

impl fmt::Display for ConnectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectorError::ConnectionError { ref reason } => {
                write!(f, "{}", reason)
            }
            ConnectorError::QueueDrop => {
                write!(f, "queue drop error")
            }
        }
    }
}

impl std::error::Error for ConnectorError {}

impl From<ConnectorError> for Box<dyn std::error::Error + Send + 'static> {
    fn from(err: ConnectorError) -> Self {
        Box::new(err)
    }
}
