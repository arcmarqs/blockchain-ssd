use tonic::{transport::{Server, Channel}, Request, Response, Status};
use to_binary::BinaryString;
use std::{io, net::SocketAddr};
mod p2p;
use p2p::protocol;

use kademlia::kademlia_client::KademliaClient;
use kademlia::{PingM,StoreReq,StoreRepl,FNodeReq,FNodeRepl,FValueReq,FValueRepl};

pub mod kademlia {
    tonic::include_proto!("kadproto");
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = KademliaClient::connect("http://0.0.0.0:50051").await?;

    let request = Request::new(
        PingM {
            cookie: "234312".to_owned(),
            id: vec![1,2,3,4],
        }
    );



    let response = client.ping(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}