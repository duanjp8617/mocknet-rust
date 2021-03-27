
use super::message_queue::{Queue, Sender};
use indradb_proto as proto;

type ConnectorResponse = Result<(Client, u64), proto::ClientError>;

enum ConnectorMessage {
    GetClient,
    ClientFail(u64),
}

// runs inside a task to do the lazy connection
struct ConnectorBackend {
    queue: Queue<ConnectorMessage, ConnectorResponse>,
}

// this is actually a sender that sends the request to the Connector Backend
// and listens for the returned message. The message is the connected client
struct Connector {
    sender: Sender<ConnectorMessage, ConnectorResponse>
}

// this is actually a wrapper for indradb-proto::proto::Client, it can be used to deliver 
// a connection signal when it is dropped
struct Client {
    client: Option<proto::Client>,
    client_id: u64,
    sender: Sender<ConnectorMessage, ConnectorResponse>
}

impl Drop for Client {
    fn drop(&mut self) {
        match self.client {
            None => {
                // notify the backend that the client fails
                let _ = self.sender.send(ConnectorMessage::ClientFail(self.client_id));
            },
            Some(_) => {}
        }
    }
}