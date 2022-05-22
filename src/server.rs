use tonic::{transport::{Server, Channel}, Request, Response, Status};
use to_binary::BinaryString;
use std::{io, net::SocketAddr};
mod p2p;
use p2p::{kad,protocol, key};


use kademlia::kademlia_server::{Kademlia,KademliaServer};

pub mod kademlia {
    tonic::include_proto!("kadproto");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
   /* 
   // kadn.print_rtable();
    let key = key::Key::new("25".to_owned() + &"1616".to_owned());
    println!("{:?}", kadn.lookup(key));
    */

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    let addr = SocketAddr::new(buffer[0..buffer.len() - 1].parse()?,50051);
    let ip = addr.ip().to_string();
    let port = addr.port();
    let mut protocol = protocol::KademliaProtocol::new(ip,port);

    for i in 0..50 {
        let k = kad::KadNode::new(i.to_string(),1616);
        protocol.node.insert(k.as_contact());
    }
    let svc = protocol.create_server();
    Server::builder()
        .add_service(svc)
        .serve(addr)
        .await?;

    Ok(())
}