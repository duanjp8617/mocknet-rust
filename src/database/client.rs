use std::{collections::HashMap, ops::Deref, ops::DerefMut, str::FromStr, sync::Arc};

use super::errors::ConnectorError;
use super::helpers;
use super::message_queue;
use super::message_queue::{Queue, Sender};
use crate::emunet::{ClusterInfo, IdAllocator, User, EMUNET_NUM_POWER};

use indradb_proto as proto;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

type ConnectorResponse = Result<(proto::Client, u64, Arc<Semaphore>), ConnectorError>;

enum ConnectorMessage {
    GetClient,
    ClientFail(u64),
}

async fn do_connect(db_addr: &str) -> Result<proto::Client, ConnectorError> {
    let endpoint = tonic::transport::Endpoint::from_str(db_addr)?;
    Ok(proto::Client::new(endpoint).await?)
}
struct ConnectorBackend {
    db_addr: String,
    client_id: u64,
    client_opt: Option<proto::Client>,
    semaphore: Arc<Semaphore>,
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
            semaphore: Arc::new(Semaphore::new(1)),
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
                            let _ = responder.send(Ok((
                                client.clone(),
                                self.client_id,
                                self.semaphore.clone(),
                            )));
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
                                    let _ = responder.send(Ok((
                                        client,
                                        self.client_id,
                                        self.semaphore.clone(),
                                    )));
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

pub(crate) struct Client {
    client: proto::Client,
    client_id: u64,
    semaphore: Arc<Semaphore>,
    sender: Sender<ConnectorMessage, ConnectorResponse>,
}

impl Client {
    pub(crate) fn notify_failure(self) {
        let _ = self
            .sender
            .send(ConnectorMessage::ClientFail(self.client_id));
    }

    pub(crate) async fn guarded_tran(&mut self) -> Result<GuardedTransaction, proto::ClientError> {
        let tran = self.client.transaction().await?;
        let singleton_guard = self.semaphore.clone().acquire_owned().await.unwrap();
        Ok(GuardedTransaction {
            tran,
            _singleton_guard: singleton_guard,
        })
    }
}

pub(crate) struct GuardedTransaction {
    tran: proto::Transaction,
    _singleton_guard: OwnedSemaphorePermit,
}

impl Deref for GuardedTransaction {
    type Target = proto::Transaction;

    fn deref(&self) -> &Self::Target {
        &self.tran
    }
}

impl DerefMut for GuardedTransaction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tran
    }
}

#[derive(Clone)]
pub struct Connector {
    sender: Sender<ConnectorMessage, ConnectorResponse>,
}

impl Connector {
    // internal interface for acquring a Client
    pub(crate) async fn connect(&self) -> Result<Client, ConnectorError> {
        let resp = self
            .sender
            .send_for_response(ConnectorMessage::GetClient)
            .await?;

        resp.map(move |(client, client_id, semaphore)| Client {
            client: client,
            client_id: client_id,
            semaphore,
            sender: self.sender.clone(),
        })
    }
}

pub async fn new_connector<S: std::convert::AsRef<str>>(
    db_addr: S,
) -> Result<Connector, ConnectorError> {
    let (sender, queue) = message_queue::create();
    let connector_backend = ConnectorBackend::new(db_addr.as_ref(), queue).await?;
    let _ = tokio::spawn(connector_backend.backend_task());
    Ok(Connector { sender })
}

pub async fn init(
    connector: &Connector,
    cluster_info: ClusterInfo,
) -> Result<Result<(), String>, proto::ClientError> {
    let mut client = connector
        .connect()
        .await
        .map_err(|_| proto::ClientError::ChannelClosed)?;
    let mut tran = client.guarded_tran().await?;

    let res = helpers::create_vertex(&mut tran, super::CORE_INFO_ID.clone()).await?;
    match res {
        true => {
            helpers::set_user_map(&mut tran, HashMap::<String, User>::new()).await?;
            helpers::set_cluster_info(&mut tran, cluster_info).await?;
            helpers::set_garbage_servesr(&mut tran, Vec::new()).await?;

            let allocator = IdAllocator::new();
            assert!(allocator.remaining() <= (2 as usize).pow(EMUNET_NUM_POWER));
            helpers::set_emunet_id_allocator(&mut tran, allocator).await?;

            Ok(Ok(()))
        }
        false => Ok(Err("database has already been initialized".to_string())),
    }
}

pub async fn init_ok(connector: &Connector) -> Result<bool, proto::ClientError> {
    let mut client = connector
        .connect()
        .await
        .map_err(|_| proto::ClientError::ChannelClosed)?;
    let mut tran = client.guarded_tran().await?;

    let res =
        helpers::get_vertex_json_value(&mut tran, super::CORE_INFO_ID.clone(), "user_map").await?;

    Ok(res.is_some())
}
