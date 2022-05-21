use tonic::{transport::Server, Request, Response, Status};
use to_binary::BinaryString;
use std::sync::Arc;
use std::{io, net::SocketAddr};
use tokio::sync::Mutex;

use super::kad::KadNode;
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
    node: KadNode,
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
            value: None,
            node: None,
        };

        Ok(Response::new(reply))
    }

    async fn find_node(&self, request: Request<FNodeReq>) -> Result<Response<FNodeRepl>,Status>{
        let con = kademlia::Contact {
            uid: vec![34,23],
            ip: String::from("192.23.23.3"),
            port: 3435,
        };
        let reply = FNodeRepl {
            cookie: String::from("10"),
            knode: vec![con],
        };

        Ok(Response::new(reply))
    }
}