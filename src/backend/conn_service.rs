use std::net::IpAddr;
use std::pin::Pin;
use std::future::Future;
use std::task::{Poll, Context};

use tonic::transport::{Channel, channel::Endpoint, Error};
use tower::Service;

pub struct ServerAddr {
    ip: IpAddr,
    port: u16,
}

impl ServerAddr {
    pub fn new(ip: &str, port: u16) -> Option<Self> {
        ip.parse::<IpAddr>().ok().map(|ip| {
            Self {
                ip,
                port
            }
        })
    }
}

pub struct ConnService();

impl Service<Vec<ServerAddr>> for ConnService {
    type Response = Vec<Channel>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Vec<ServerAddr>) -> Self::Future {        
        async fn make_conns(addrs: Vec<ServerAddr>) -> Result<Vec<Channel>, Error> {
            let mut vec = Vec::new();
            for addr in &addrs {
                let s_addr = "http://".to_string() + &addr.ip.to_string() +  ":" + &addr.port.to_string();
                let chan = Endpoint::new(s_addr)?.connect().await?;
                vec.push(chan);
            }

            Ok(vec)
        }
        
        Box::pin(make_conns(req))
    }
}

