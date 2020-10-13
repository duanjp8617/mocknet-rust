use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}
use mocknet::backend::conn_service;

use tower::Service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cs = conn_service::ConnService{};

    let mut addrs = Vec::new();
    addrs.push(conn_service::ServerAddr::new("127.0.0.1", 10240).unwrap());
    addrs.push(conn_service::ServerAddr::new("127.0.0.1", 10241).unwrap());
    
    let res = cs.call(addrs).await?;
    
    for chan in res {
        let mut client = GreeterClient::new(chan);
        let request = tonic::Request::new(HelloRequest {
            name: "Tonic".into(),
        });
    
        let response = client.say_hello(request).await?;
    
        println!(" wtf RESPONSE={:?}", response);
    }
    
    Ok(())
}