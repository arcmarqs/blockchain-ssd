use tonic::{transport::{Server, Channel}, Request, Response, Status};
use to_binary::BinaryString;
use std::{io, net::SocketAddr};
mod p2p;
use p2p::{
    kad,
};

use kademlia::kademlia_client::KademliaClient;
use kademlia::{PingM,StoreReq,StoreRepl,FNodeReq,FNodeRepl,FValueReq,FValueRepl,Kcontact};

pub mod kademlia {
    tonic::include_proto!("kadproto");
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = KademliaClient::connect("http://0.0.0.0:50051").await?;
    let k = kad::KadNode::new("25".to_owned(),1616);
    let n = Kcontact {
        uid: k.uid.as_bytes().to_owned(),
        ip: k.ip,
        port: k.port as i32,
    };

    println!("look for  {:?}",n.uid);

    let request = Request::new(
        FNodeReq {
            cookie: "234312".to_owned(),
            node: Some(n),
        }
    );



    let response = client.find_node(request).await?.into_inner();

    println!("RESPONSE={:?} len = {:?}", response, response.knode.len());

    Ok(())
}