use tonic::{transport::{Server, Channel}, Request, Response, Status};
use to_binary::BinaryString;
use std::{io, net::SocketAddr, sync::Arc, collections::HashSet};
use super::{
    kad::KadNode,
    node::Contact
};
use rand::Rng;
use kademlia::kademlia_client::KademliaClient;
use kademlia::{PingM,StoreReq,StoreRepl,FNodeReq,FNodeRepl,FValueReq,FValueRepl,Kcontact};

pub mod kademlia {
    tonic::include_proto!("kadproto");
}

#[derive(Debug,Default,Clone)]
pub struct Client {}

impl Client {
    pub async fn send_ping(mynode: Arc<KadNode>,contact: Contact) {
        let addr = format_address(contact.ip,contact.port);
        let mut client = KademliaClient::connect(addr).await.unwrap();  
        let mut rng = rand::thread_rng();
        let cookie: usize = rng.gen();
        let request = Request::new(
            PingM {
                cookie: cookie.to_string(),
                id : mynode.uid.as_bytes().to_owned(),
            }
        );

        println!("here");

        let response = client.ping(request).await.unwrap();
        println!("{:?}", response.into_inner());
    }

    async fn send_fnode(mynode: Arc<KadNode>, contact: Contact) -> Result<(), Box<dyn std::error::Error>> {
        let k_closest = mynode.lookup(contact.uid);
        let visited_nodes = HashSet::<Contact>::new();
        
        let addr = format_address(contact.ip,contact.port);
        let client = KademliaClient::connect(addr).await?;  
        todo!();
    }

    async fn send_fvalue(mynode: Arc<KadNode>,contact: Contact) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format_address(contact.ip,contact.port);
        let client = KademliaClient::connect(addr).await?;  
        todo!();
    }

    async fn send_store(mynode: Arc<KadNode>,contact: Contact) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format_address(contact.ip,contact.port);
        let client = KademliaClient::connect(addr).await?;
        todo!();
    }

}

fn format_address(ip: String, port: u16) -> String {
    ("http://".to_owned() + &ip + ":" + &port.to_string()).to_owned()
}
