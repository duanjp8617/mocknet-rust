use std::sync::Mutex;

use tonic::{transport::Server, Request, Response, Status};

use mocknet::k8s_api::mocknet_server::{Mocknet, MocknetServer};
use mocknet::k8s_api::*;

#[derive(Debug)]
pub struct MockServer {
    pods: Mutex<Vec<(String, String)>>,
}

impl MockServer {
    pub fn new() -> Self {
        Self {
            pods: Mutex::new(Vec::new()),
        }
    }
}

#[tonic::async_trait]
impl Mocknet for MockServer {
    async fn init(&self, request: Request<EmunetReq>) -> Result<Response<EmunetResp>, Status> {
        let inner = request.into_inner();
        println!("---------Got a new init request---------");
        println!("{:?}", &inner);

        let mut guard = self.pods.lock().unwrap();
        let reply = if guard.len() > 0 {
            EmunetResp { status: false }
        } else {
            let mut ip_addr: u32 = std::net::Ipv4Addr::from([10, 0, 0, 0]).into();
            let pods = inner.pods;
            for pod in pods {
                guard.push((
                    pod.metadata.unwrap().name,
                    std::net::Ipv4Addr::from(ip_addr).to_string(),
                ));
                ip_addr += 1
            }

            EmunetResp { status: true }
        };

        Ok(Response::new(reply))
    }

    async fn delete(&self, request: Request<EmunetReq>) -> Result<Response<EmunetResp>, Status> {
        let inner = request.into_inner();
        println!("---------Got a new delete request---------");
        println!("{:?}", &inner);

        let mut guard = self.pods.lock().unwrap();
        let reply = if guard.len() == 0 {
            EmunetResp { status: false }
        } else if guard.len() != inner.pods.len() {
            EmunetResp { status: false }
        } else {
            let mut hashset = std::collections::HashSet::new();
            let _: Vec<_> = inner
                .pods
                .into_iter()
                .map(|pod| hashset.insert(pod.metadata.unwrap().name))
                .collect();

            let res: Option<Vec<_>> = guard
                .iter()
                .map(|(pod_name, _)| {
                    if hashset.get(pod_name).is_some() {
                        Some(())
                    } else {
                        None
                    }
                })
                .collect();
            match res {
                Some(_) => {
                    guard.clear();
                    EmunetResp { status: true }
                }
                _ => EmunetResp { status: false },
            }
        };

        Ok(Response::new(reply))
    }

    async fn query(&self, request: Request<QueryReq>) -> Result<Response<QueryResp>, Status> {
        let inner = request.into_inner();
        println!("---------Got a new query request---------");
        println!("{:?}", &inner);

        let is_init = inner.is_init;

        let reply = match is_init {
            false => {
                let guard = self.pods.lock().unwrap();
                if guard.len() == 0 {
                    QueryResp {
                        status: true,
                        device_infos: Vec::new(),
                    }
                } else {
                    QueryResp {
                        status: false,
                        device_infos: Vec::new(),
                    }
                }
            }
            true => {
                let guard = self.pods.lock().unwrap();
                if guard.len() == 0 {
                    QueryResp {
                        status: false,
                        device_infos: Vec::new(),
                    }
                } else {
                    let mut hashset = std::collections::HashSet::new();
                    for pod_name in inner.pod_names {
                        hashset.insert(pod_name);
                    }

                    if guard.len() == hashset.len() {
                        let device_infos: Option<Vec<DeviceInfo>> = guard
                            .iter()
                            .map(|(pod_name, login_ip)| {
                                if hashset.get(pod_name).is_some() {
                                    Some(DeviceInfo {
                                        pod_name: pod_name.clone(),
                                        login_ip: login_ip.clone(),
                                        username: "fuck".to_string(),
                                        password: "fuck".to_string(),
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect();
                        match device_infos {
                            Some(inner) => QueryResp {
                                status: true,
                                device_infos: inner,
                            },
                            None => QueryResp {
                                status: false,
                                device_infos: Vec::new(),
                            },
                        }
                    } else {
                        QueryResp {
                            status: false,
                            device_infos: Vec::new(),
                        }
                    }
                }
            }
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:3032".parse()?;
    let mockserver = MockServer::new();

    Server::builder()
        .add_service(MocknetServer::new(mockserver))
        .serve(addr)
        .await?;

    Ok(())
}
