use tonic::{transport::{Server, Channel}, Request, Response, Status};
use to_binary::BinaryString;
use std::{io, net::SocketAddr, sync::Arc};
use super::{kad::KadNode,protocol, key};
use kademlia::{
    kademlia_server::{Kademlia,KademliaServer},
    kademlia_client::KademliaClient, 
    PingM,StoreReq,StoreRepl,FNodeReq,FNodeRepl,FValueReq,FValueRepl
};
use tokio::task;
use tokio::runtime::Handle;

pub mod kademlia {
    tonic::include_proto!("kadproto");
}


pub async fn server(addr: SocketAddr, node: Arc<KadNode>) {
   /* 
   // kadn.print_rtable();
    let key = key::Key::new("25".to_owned() + &"1616".to_owned());
    println!("{:?}", kadn.lookup(key));
    */
    //let ip = buffer.split(':').collect();
    let protocol = protocol::KademliaProtocol::new(node);
    println!("node: {:?}",protocol.node);
    protocol.node.print_rtable().await;

    /* for i in 0..50 {
        let k = kad::KadNode::new(i.to_string(),1616);
        protocol.node.insert(k.as_contact());
    } */
    let svc = protocol.create_server();

    Server::builder()
        .add_service(svc)
        .serve(addr).await.unwrap();
}