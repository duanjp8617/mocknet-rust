use tonic::{transport::Server, Request, Response, Status};

use mocknet::k8s_api::mocknet_server::{Mocknet, MocknetServer};
use mocknet::k8s_api::*;

#[derive(Debug, Default)]
pub struct MockServer {
    pods: Vec<(String, String)>
}

#[tonic::async_trait]
impl Mocknet for MockServer {
    async fn init(
        &self,
        request: Request<EmunetReq>, 
    ) -> Result<Response<EmunetResp>, Status> { 
        let inner = request.into_inner();
        println!("Got a request: {:?}", &inner);

        let reply = EmunetResp {
           status: true
        };

        Ok(Response::new(reply)) 
    }

    async fn delete(
        &self,
        request: Request<EmunetReq>,
    ) -> Result<Response<EmunetResp>, Status> {
        let inner = request.into_inner();
        println!("Got a request: {:?}", &inner);

        let reply = EmunetResp {
           status: true
        };

        Ok(Response::new(reply)) 
    }

    async fn query(
        &self,
        request: Request<QueryReq>, 
    ) -> Result<Response<QueryResp>, Status> {
        let inner = request.into_inner();
        println!("Got a request: {:?}", &inner);

        let reply = QueryResp {
           status: true,
           device_infos: Vec::new()
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:3032".parse()?;
    let mockserver = MockServer::default();

    Server::builder()
        .add_service(MocknetServer::new(mockserver))
        .serve(addr)
        .await?;

    Ok(())
}