use tonic::{transport::Server, Request, Response, Status};
mod p2p;
use p2p::kad as kad;
use p2p::node as nd;
use p2p::key as key;
use p2p::rtable as rt;
use to_binary::BinaryString;

use kademlia::kademlia_server::{Kademlia,KademliaServer};
use kademlia::{PingM,StoreReq,StoreRepl,FNodeReq,FNodeRepl,FValueReq,FValueRepl};

pub mod kademlia {
    tonic::include_proto!("kadproto");
}

pub struct KademliaProtocol{}

#[tonic::async_trait]
impl Kademlia for KademliaProtocol {
    async fn ping(&self, request: Request<PingM>) -> Result<Response<PingM>,Status>{
        let reply = PingM {
            cookie: String::from("10"),
            id: vec![87,252,234],
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ip = String::from("127.0.0.");
    let port = 5050;
    let mut origin = kad::KadNode::new(String::from("192.0.0.1"),port);
    for i in 0..20 as i32 {
        let insertip = ip.clone() + &i.to_string();
        let k = kad::KadNode::new(insertip,port);
        origin.insert(k.as_contact());
    } 

    let insertip = String::from("192.0.0.2");
    let k = kad::KadNode::new(insertip,port);
    let look = key::Key::new(String::from("127.0.0.1"));

    origin.insert(k.as_contact());
    //origin.print_rtable();
    println!("{:?}",origin.lookup(look).bucket.as_ref().unwrap().len());
    println!("{:?}",origin.lookup(k.uid).bucket.as_ref().unwrap().len());
    
    Ok(())
}
