use rand::Rng;
use tonic::{transport::Server, Request, Response, Status};
use to_binary::BinaryString;
use std::collections::HashSet;
use std::sync::Arc;
use std::{io, net::SocketAddr};
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

#[derive(Debug,Clone)]
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

    fn lookup(&self, key: Key) -> Vec<Kcontact> {
        let k_closest_boxed = self.node.lookup(key);
        let mut k_closest = Vec::with_capacity(k_closest_boxed.len());

        for k in k_closest_boxed {
            
            let kc = k.as_kcontact();
            k_closest.push(kc);
        }

        k_closest
    }

    fn insert_update(&self,id: Vec<u8>, remote_addr: SocketAddr) {
        let con = Contact::new(Key::from_vec(id), remote_addr.ip().to_string(),remote_addr.port());
        self.node.insert(con);
    }
}

#[tonic::async_trait]
impl Kademlia for KademliaProtocol {
   async fn ping(&self, request: Request<PingM>) -> Result<Response<PingM>,Status>{
        if let Some(sender_addr) = request.remote_addr() {
            println!("{:?}",sender_addr);
        };
        let remote_addr = request.remote_addr().unwrap();
        let req = request.into_inner();
        self.insert_update(req.id,remote_addr);
        let uid = self.node.uid.as_bytes().clone();
        let reply = PingM {
            cookie: req.cookie,
            id: uid.to_owned(),
        };
        println!("Sending reply: {:?}", reply);
        Ok(Response::new(reply))
    }

    async fn store(&self, request: Request<StoreReq>) -> Result<Response<StoreRepl>,Status>{
        let remote_addr = request.remote_addr().unwrap();
        let req = request.into_inner();
        let key_bytes = req.key;
        let key = Key::from_vec(key_bytes);
        self.insert_update(req.my_id,remote_addr);
        self.node.store_value(key, req.value);
        let reply = StoreRepl {
            cookie: req.cookie,
            my_id: self.node.uid.as_bytes().to_owned(),
        };

        Ok(Response::new(reply))
    }

    async fn find_value(&self, request: Request<FValueReq>) -> Result<Response<FValueRepl>,Status>{
        let remote_addr = request.remote_addr().unwrap();
        let req = request.into_inner();
        let key_bytes = req.uid;
        let lookup_key = Key::from_vec(key_bytes);
        self.insert_update(req.my_id,remote_addr);
        let mut value: Option<String> = None;
        let mut k_closest = Vec::new();
        match self.node.retrieve(lookup_key) {
            Some(val) => value = Some(val),
            None => k_closest = self.lookup(lookup_key),
        };

        let reply = FValueRepl {
            cookie: req.cookie,
            my_id: self.node.uid.as_bytes().to_owned(),
            value: value,
            node: k_closest,
        };

        Ok(Response::new(reply))
    }

    async fn find_node(&self, request: Request<FNodeReq>) -> Result<Response<FNodeRepl>,Status>{
        let remote_addr = request.remote_addr().unwrap();
        let req = request.into_inner();
        let key_bytes = req.uid;
        let lookup_key = Key::from_vec(key_bytes);
        let k = self.lookup(lookup_key);
        self.insert_update(req.my_id,remote_addr);
        let reply = FNodeRepl {
            cookie: req.cookie,
            my_id: self.node.uid.as_bytes().to_owned(),
            knode: k,
        };

        Ok(Response::new(reply))
    }
}