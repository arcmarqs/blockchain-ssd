use tonic::{transport::{Server, Channel}, Request, Response, Status};
use to_binary::BinaryString;
use std::{io, net::SocketAddr, sync::Arc, collections::HashSet};
mod p2p;
use p2p::{
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
pub struct Client {
    node: Arc<KadNode>,
}

impl Client {
    pub fn bind_node(&mut self, node: &Arc<KadNode>) -> Client {
        Client {
            node : Arc::clone(node),
        }
    }

    pub async fn send_ping(&self,contact: Contact) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format_address(contact.ip,contact.port);
        let mut client = KademliaClient::connect(addr).await?;  
        let mut rng = rand::thread_rng();
        let cookie: usize = rng.gen();
        let request = Request::new(
            PingM {
                cookie: cookie.to_string(),
                id : self.node.uid.as_bytes().to_owned(),
            }
        );

        let response = client.ping(request).await?;
        println!("{:?}", response.into_inner());
        Ok(())
    }

    async fn send_fnode(&self,contact: Contact) -> Result<(), Box<dyn std::error::Error>> {
        let k_closest = self.node.lookup(contact.uid);
        let visited_nodes = HashSet::<Contact>::new();
        
        let addr = format_address(contact.ip,contact.port);
        let client = KademliaClient::connect(addr).await?;  
        todo!();
    }

    async fn send_fvalue(&self,contact: Contact) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format_address(contact.ip,contact.port);
        let client = KademliaClient::connect(addr).await?;  
        todo!();
    }

    async fn send_store(&self,contact: Contact) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format_address(contact.ip,contact.port);
        let client = KademliaClient::connect(addr).await?;
        todo!();
    }

}

fn format_address(ip: String, port: u16) -> String {
    ("http://".to_owned() + &ip + ":" + &port.to_string()).to_owned()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50050";
    let mut kclient = Client::default();
    let node = Arc::new(KadNode::new(addr.to_string(),0));
    kclient.bind_node(&node);

    Ok(())
}