use tonic::{transport::{Server, Channel}, Request, Response, Status};
use to_binary::BinaryString;
use std::{io, net::SocketAddr};
mod p2p;
use p2p::{
    protocol,
};


use kademlia::{ 
    kademlia_server::{Kademlia,KademliaServer},
    kademlia_client::KademliaClient, 
    PingM
};

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

    /* 
    let ip = String::from("127.0.0.");
    let port = 5050;
    let mut origin = kad::KadNode::new(String::from("192.0.0.1"),port);
    for i in 0..20 as i32 {
        let insertip = ip.clone() + &i.to_string();
        let k = kad::KadNode::new(insertip,port);
        origin.insert(k.as_contact());
    } 

    let insertip = String::from("192.0.0.2");
    let k = kad::KadNode::new(insertip,port);
    let look = key::Key::new(String::from("127.0.0.1"));

    origin.insert(k.as_contact());
    //origin.print_rtable();
    println!("{:?}",origin.lookup(look).bucket.as_ref().unwrap().len());
    println!("{:?}",origin.lookup(k.uid).bucket.as_ref().unwrap().len());
    */

    Ok(())
}
