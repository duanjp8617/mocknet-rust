use tonic::{transport::Server, Request, Response, Status};

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};

use dummy_cli_parser::{CliParser, PatternType};
use std::net::{IpAddr};

pub mod hello_world {
    tonic::include_proto!("helloworld"); // The string specified here must match the proto package name
}

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<HelloReply>, Status> { // Return an instance of type HelloReply
        println!("Got a request: {:?}", request);

        let reply = hello_world::HelloReply {
            message: format!("Hello {}!", request.into_inner().name).into(), // We must use .into_inner() as the fields of gRPC requests and responses are private
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}

struct ParseObj {
    addr: IpAddr,
    port: i32,
}

fn parse_cli() -> Result<ParseObj, String> {
    let mut parser = CliParser::new(ParseObj{
        addr: "127.0.0.1".parse::<IpAddr>().unwrap(),
        port: 1024,
    });

    parser.register_pattern("-ip", PatternType::OptionalWithArg, "ip address", 
        |arg_str, parse_obj| {
            arg_str.parse::<IpAddr>().map(|addr|{
                parse_obj.addr = addr;
            }).map_err(|_|{
                String::from(format!("fail to parse argument \"{}\"", &arg_str))
            })
        }
    )?;

    parser.register_pattern("-p", PatternType::OptionalWithArg, "port", 
        |arg_str, parse_obj| {
            let parse_res = arg_str.parse::<i32>();
            if parse_res.is_ok() {
                let port = parse_res.unwrap();
                if port >=0 && port <= 65535 {
                    parse_obj.port = port;
                    Ok(())
                }
                else {
                    Err(String::from(format!("port number {} is invalid", &port)))
                }
            }
            else {
                Err(String::from(format!("fail to parse argument \"{}\"", &arg_str)))
            }
        }
    )?;

    parser.parse_env_args()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parse_obj = parse_cli()?;

    let addr = (parse_obj.addr.to_string() + ":" + &parse_obj.port.to_string()).parse()?;
    let greeter = MyGreeter::default();
    println!("ready to run!");
    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}