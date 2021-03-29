use std::{collections::HashMap, str::FromStr};

use super::errors::ConnectorError;
use super::helpers;
use super::message_queue;
use super::message_queue::{Queue, Sender};
use crate::new_emunet::cluster::ClusterInfo;
use crate::new_emunet::user::User;

// use indradb::Type;
// use indradb::{BulkInsertItem, Transaction};
// use indradb::{RangeVertexQuery, SpecificVertexQuery, VertexQueryExt};
// use indradb::{Vertex, VertexQuery};
// use indradb::{VertexProperty, VertexPropertyQuery};
use indradb_proto as proto;
// use proto::ClientError;
// use uuid::Uuid;

type ConnectorResponse = Result<(proto::Client, u64), ConnectorError>;
pub type QueryResult<T> = Result<T, String>;

macro_rules! succeed {
    ($arg: expr) => {
        Ok(Ok($arg))
    };
}

macro_rules! fail {
    ($s: expr) => {
        Ok(Err($s))
    };
}

enum ConnectorMessage {
    GetClient,
    ClientFail(u64),
}

pub async fn new_connector(db_addr: &str) -> Result<Connector, ConnectorError> {
    let (sender, queue) = message_queue::create();
    let connector_backend = ConnectorBackend::new(db_addr, queue).await?;
    let _ = tokio::spawn(connector_backend.backend_task());
    Ok(Connector { sender })
}

async fn do_connect(db_addr: &str) -> Result<proto::Client, ConnectorError> {
    let endpoint = tonic::transport::Endpoint::from_str(db_addr)?;
    Ok(proto::Client::new(endpoint).await?)
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
    ) -> Result<Self, ConnectorError> {
        let client = do_connect(db_addr).await?;
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
                Some((msg, responder)) => match msg {
                    ConnectorMessage::GetClient => match self.client_opt {
                        Some(ref client) => {
                            let _ = responder.send(Ok((client.clone(), self.client_id)));
                        }
                        None => {
                            let res = tokio::time::timeout(
                                std::time::Duration::from_millis(500),
                                do_connect(&self.db_addr),
                            )
                            .await;
                            match res {
                                Err(err) => {
                                    let _ = responder.send(Err(err.into()));
                                }
                                Ok(Err(err)) => {
                                    let _ = responder.send(Err(err));
                                }
                                Ok(Ok(client)) => {
                                    self.client_opt = Some(client.clone());
                                    self.client_id += 1;
                                    let _ = responder.send(Ok((client, self.client_id)));
                                }
                            }
                        }
                    },
                    ConnectorMessage::ClientFail(client_id) => {
                        if client_id == self.client_id && self.client_opt.is_some() {
                            self.client_opt = None
                        }
                    }
                },
            }
        }
    }
}

// this is actually a sender that sends the request to the Connector Backend
// and listens for the returned message. The message is the connected client
#[derive(Clone)]
pub struct Connector {
    sender: Sender<ConnectorMessage, ConnectorResponse>,
}

impl Connector {
    pub async fn connect(&self) -> Result<Client, ConnectorError> {
        let resp = self
            .sender
            .send_for_response(ConnectorMessage::GetClient)
            .await?;

        resp.map(move |(client, client_id)| Client {
            client: client,
            client_id: client_id,
            sender: self.sender.clone(),
        })
    }
}

// this is actually a wrapper for indradb-proto::proto::Client, it can be used to deliver
// a connection signal when it is dropped
pub struct Client {
    client: proto::Client,
    client_id: u64,
    sender: Sender<ConnectorMessage, ConnectorResponse>,
}

// impl Drop for Client {
//     fn drop(&mut self) {
//         match self.client {
//             None => {
//                 // notify the backend that the client fails
//                 let _ = self
//                     .sender
//                     .send(ConnectorMessage::ClientFail(self.client_id));
//             }
//             Some(_) => {}
//         }
//     }
// }

// public interfaces
impl Client {
    pub async fn init(
        &mut self,
        cluster_info: ClusterInfo,
    ) -> Result<QueryResult<()>, proto::ClientError> {
        let mut tran = self.tran().await?;

        let res = helpers::create_vertex(&mut tran, Some(super::CORE_INFO_ID.clone())).await?;
        match res {
            Some(_) => {
                helpers::set_user_map(&mut tran, HashMap::<String, User>::new()).await?;
                helpers::set_cluster_info(&mut tran, cluster_info).await?;

                succeed!(())
            }
            None => fail!("database has already been initialized".to_string()),
        }
    }

    pub async fn tran(&mut self) -> Result<proto::Transaction, proto::ClientError> {
        self.client.transaction().await
    }

    pub fn notify_failure(self) {
        let _ = self
            .sender
            .send(ConnectorMessage::ClientFail(self.client_id));
    }
}
