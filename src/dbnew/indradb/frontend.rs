
use crate::dbnew::errors::BackendError;

use super::message_queue;
use super::message::{Request, Response};

pub struct FrontEnd {
    sender: message_queue::Sender<Request, Response, BackendError>,
}

impl FrontEnd {
    pub fn new(sender: message_queue::Sender<Request, Response, BackendError>) -> Self {
        Self {sender}
    }
}

impl Clone for FrontEnd {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone()
        }
    }
}

impl FrontEnd {
    pub async fn init(&self) -> Result<(), BackendError> {
        let _ = self.sender.send(Request::Init).await?;
        Ok(())
    }
}