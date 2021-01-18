pub mod hello_world {
    tonic::include_proto!("helloworld");
}

use std::iter::Iterator;

use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;

pub struct ContainerServerAddr(String);

pub struct ContainerBackendBuilder<I> {
    addr_iter: I,
}

impl<I> ContainerBackendBuilder<I>
where
    I: Iterator<Item = ContainerServerAddr>,
{
    pub fn new(iter: I) -> Self {
        Self { addr_iter: iter }
    }

    pub async fn build(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for addr in &mut self.addr_iter {
            let mut server_client = GreeterClient::connect(addr.0).await?;

            let request = tonic::Request::new(HelloRequest {
                name: "Tonic".into(),
            });

            let _ = server_client.say_hello(request).await?;

            println!("Get the fucking resposne");
        }

        Ok(())
    }
}
