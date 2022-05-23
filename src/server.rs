use tonic::{transport::{Server, Channel}, Request, Response, Status};
use to_binary::BinaryString;
use std::{io, net::SocketAddr, sync::Arc};
mod p2p;
use p2p::{kad::{self, KadNode},protocol, key};
use kademlia::{
    kademlia_server::{Kademlia,KademliaServer},
    kademlia_client::KademliaClient, 
    PingM,StoreReq,StoreRepl,FNodeReq,FNodeRepl,FValueReq,FValueRepl
};

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
    let addr: SocketAddr = buffer[0..buffer.len() - 1].parse()?;
    println!("{:?}",addr);
    //let ip = buffer.split(':').collect();
    let protocol = protocol::KademliaProtocol::new(addr.to_string(),0);
    println!("{:?}",protocol.node);
    /* for i in 0..50 {
        let k = kad::KadNode::new(i.to_string(),1616);
        protocol.node.insert(k.as_contact());
    } */
    let svc = protocol.create_server();
    Server::builder()
        .add_service(svc)
        .serve(addr)
        .await?;

    Ok(())
}