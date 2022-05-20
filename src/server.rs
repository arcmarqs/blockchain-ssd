use tonic::{transport::{Server, Channel}, Request, Response, Status};
use to_binary::BinaryString;
use std::{io, net::SocketAddr};
mod p2p;
use p2p::protocol;


use kademlia::kademlia_server::{Kademlia,KademliaServer};

pub mod kademlia {
    tonic::include_proto!("kadproto");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    let addr = SocketAddr::new(buffer[0..buffer.len() - 1].parse()?,50051);
    let ip = addr.ip().to_string();
    let port = addr.port();
    let protocol = protocol::KademliaProtocol::new(ip,port);
    let svc = protocol::create_server(protocol);
    Server::builder()
        .add_service(svc)
        .serve(addr)
        .await?;

    Ok(())
}