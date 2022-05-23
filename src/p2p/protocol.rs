use rand::Rng;
use tonic::{transport::Server, Request, Response, Status};
use to_binary::BinaryString;
use std::sync::Arc;
use std::{io, net::SocketAddr};
use tokio::sync::Mutex;


use self::kademlia::Kcontact;

use super::node::Contact;
use super::{
    kad::KadNode,
    key::Key
};

use kademlia::{ 
    kademlia_server::{Kademlia,KademliaServer},
    kademlia_client::KademliaClient, 
    PingM,StoreReq,StoreRepl,FNodeReq,FNodeRepl,FValueReq,FValueRepl
};

pub mod kademlia {
    tonic::include_proto!("kadproto");
}

#[derive(Debug, Default,Clone)]
pub struct KademliaProtocol{
    pub node: KadNode,
}

impl KademliaProtocol {
    pub fn new(ip: String, port: u16) -> KademliaProtocol {
        KademliaProtocol {
            node : KadNode::new(ip,port),
        }
    }

    pub fn create_server(self) -> KademliaServer<KademliaProtocol> {
        KademliaServer::<KademliaProtocol>::new(self)
    }

    pub fn lookup(&self, key: Key) -> Vec<Kcontact> {
        let k_closest_boxed = self.node.lookup(key);
        let mut k_closest = Vec::with_capacity(k_closest_boxed.len());

        for k in k_closest_boxed {
            
            let kc = k.as_kcontact();
            k_closest.push(kc);
        }

        k_closest
    }

    async fn send_ping(&self,addr:String) -> Result<(), Box<dyn std::error::Error>> {
        let client = KademliaClient::connect(addr).await?;  
        let mut rng = rand::thread_rng();
        let cookie: usize = rng.gen();
        let request = Request::new(
            PingM {
                cookie: cookie.to_string(),
                id : self.node.uid.as_bytes().to_owned(),
            }
        );

        Ok(())
    }

    async fn send_fnode(&self,addr:String) -> Result<(), Box<dyn std::error::Error>> {
        let client = KademliaClient::connect(addr).await?;  
        todo!();
    }

    async fn send_fvalue(&self,addr:String) -> Result<(), Box<dyn std::error::Error>> {
        let client = KademliaClient::connect(addr).await?;  
        todo!();
    }

    async fn send_store(&self,addr:String) -> Result<(), Box<dyn std::error::Error>> {
        let client = KademliaClient::connect(addr).await?;
        todo!();
    }

}

#[tonic::async_trait]
impl Kademlia for KademliaProtocol {
   async fn ping(&self, request: Request<PingM>) -> Result<Response<PingM>,Status>{
        if let Some(sender_addr) = request.remote_addr() {
            println!("{:?}",sender_addr);
        }

        let req = request.into_inner();
        let uid = self.node.uid.as_bytes().clone();
        let reply = PingM {
            cookie: req.cookie,
            id: uid.to_owned(),
        };
        
        Ok(Response::new(reply))
    }

    async fn store(&self, request: Request<StoreReq>) -> Result<Response<StoreRepl>,Status>{
        let reply = StoreRepl {
            cookie: String::from("10"),
        };

        Ok(Response::new(reply))
    }

    async fn find_value(&self, request: Request<FValueReq>) -> Result<Response<FValueRepl>,Status>{
        let reply = FValueRepl {
            cookie: String::from("10"),
            value: "placeholder".to_owned(),
            node: None,
        };

        Ok(Response::new(reply))
    }

    async fn find_node(&self, request: Request<FNodeReq>) -> Result<Response<FNodeRepl>,Status>{
        let req = request.into_inner();
        let key_bytes = req.node.unwrap().uid;
        let lookup_key = Key::from_vec(key_bytes);
        let k = self.lookup(lookup_key);
        println!("reply: {:?}", k);

        let reply = FNodeRepl {
            cookie: req.cookie,
            knode: k,
        };

        Ok(Response::new(reply))
    }
}