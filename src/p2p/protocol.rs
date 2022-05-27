use rand::Rng;
use tonic::{transport::Server, Request, Response, Status};
use to_binary::BinaryString;
use std::collections::HashSet;
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
    pub node: Arc<KadNode>,
}

impl KademliaProtocol {
    pub fn new(node: Arc<KadNode>) -> KademliaProtocol {
        KademliaProtocol {
            node
        }
    }

    pub fn create_server(self) -> KademliaServer<KademliaProtocol> {
        KademliaServer::<KademliaProtocol>::new(self)
    }

    async fn lookup(&self, key: Key) -> Vec<Kcontact> {
        let k_closest_boxed = self.node.lookup(key).await;
        let mut k_closest = Vec::with_capacity(k_closest_boxed.len());

        for k in k_closest_boxed {
            
            let kc = k.as_kcontact();
            k_closest.push(kc);
        }

        k_closest
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
        println!("Sending reply: {:?}", reply);
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
        let key_bytes = req.uid;
        let lookup_key = Key::from_vec(key_bytes);
        let k = self.lookup(lookup_key).await;
        println!("reply: {:?}", k);

        let reply = FNodeRepl {
            cookie: req.cookie,
            knode: k,
        };

        Ok(Response::new(reply))
    }
}