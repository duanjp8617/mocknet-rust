use std::str::FromStr;

use super::{
    message,
    message_queue::{Queue, Sender},
};
use indradb_proto as proto;

use super::errors::ConnError;
use super::message_queue;

type ConnectorResponse = Result<(proto::Client, u64), ConnError>;

enum ConnectorMessage {
    GetClient,
    ClientFail(u64),
}

async fn new_connector(db_addr: String) -> Result<Connector, ConnError> {
    let (sender, queue) = message_queue::create();
    let connector_backend = ConnectorBackend::new(&db_addr, queue).await?;
    Ok(Connector { sender })
}

// runs inside a task to do the lazy connection
struct ConnectorBackend {
    db_addr: String,
    client_id: u64,
    client_opt: Option<proto::Client>,
    queue: Queue<ConnectorMessage, ConnectorResponse>,
}

impl ConnectorBackend {
    async fn new(
        db_addr: &str,
        queue: Queue<ConnectorMessage, ConnectorResponse>,
    ) -> Result<Self, ConnError> {
        let endpoint = tonic::transport::Endpoint::from_str(db_addr)?;
        let client = proto::Client::new(endpoint).await?;
        Ok(Self {
            db_addr: db_addr.to_string(),
            client_id: 1,
            client_opt: Some(client),
            queue,
        })
    }

    async fn backend_task(mut self) {
        loop {
            let msg = self.queue.recv().await;
            match msg {
                None => break,
                Some((msg, responder)) => {
                    match msg {
                        ConnectorMessage => {
                            
                        }
                    }
                }
            }
        }
    }
}

// this is actually a sender that sends the request to the Connector Backend
// and listens for the returned message. The message is the connected client
#[derive(Clone)]
struct Connector {
    sender: Sender<ConnectorMessage, ConnectorResponse>,
}

impl Connector {
    pub async fn connect(self) -> Result<Client, ConnError> {
        let resp = self
            .sender
            .send_for_response(ConnectorMessage::GetClient)
            .await?;

        resp.map(move |(client, client_id)| Client {
            client: Some(client),
            client_id: client_id,
            sender: self.sender,
        })
    }
}

// this is actually a wrapper for indradb-proto::proto::Client, it can be used to deliver
// a connection signal when it is dropped
struct Client {
    client: Option<proto::Client>,
    client_id: u64,
    sender: Sender<ConnectorMessage, ConnectorResponse>,
}

impl Drop for Client {
    fn drop(&mut self) {
        match self.client {
            None => {
                // notify the backend that the client fails
                let _ = self
                    .sender
                    .send(ConnectorMessage::ClientFail(self.client_id));
            }
            Some(_) => {}
        }
    }
}
