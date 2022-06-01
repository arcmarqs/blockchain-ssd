use std::{sync::Arc, net::SocketAddr};

use chrono::Utc;
use prost::Message;
use tonic::{Request, Response, Status, Code};

use super::{kad::KadNode, 
    key::NodeID, 
    node::{Contact}, 
    signatures::Signer, 
    kademlia::{
        kademlia_server::{Kademlia, KademliaServer}, 
        PingM, Kcontact, StoreReq, StoreRepl, FValueReq, FValueRepl, 
        f_value_repl::{HasValue::{Value,Node as HNode}, HasValue},
        Kclosest, Header, FNodeReq, FNodeRepl}};

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

    fn lookup(&self, key: NodeID) -> Vec<Kcontact> {
        let k_closest_boxed = self.node.lookup(key);
        let mut k_closest = Vec::with_capacity(k_closest_boxed.len());

        for k in k_closest_boxed {
            
            let kc = k.as_kcontact();
            k_closest.push(kc);
        }

        k_closest
    }

    fn insert_update(&self,id: Vec<u8>,pub_key: &[u8], remote_addr: SocketAddr) {
        let con = Contact::new(NodeID::from_vec(id), remote_addr.to_string(), pub_key.to_vec());
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
        let header = req.header.unwrap();
        self.insert_update(header.my_id,&header.pub_key,remote_addr);
        let timestamp = Utc::now().timestamp();
        let reply = PingM {
            cookie: req.cookie,
            header: Some( Header {
                my_id: self.node.uid.as_bytes().to_owned(),
                pub_key: self.node.get_pubkey(),
                nonce: self.node.get_nonce(),
                timestamp,
                signature: Signer::sign_weak_header_rep(timestamp,&header.pub_key, &self.node.address, &header.signature),
            }),
        };
        println!("Sending reply: {:?}", reply);
        Ok(Response::new(reply))
    }

    async fn store(&self, request: Request<StoreReq>) -> Result<Response<StoreRepl>,Status>{
        let remote_addr = request.remote_addr().unwrap();
        let req = request.into_inner();
        let key_bytes = req.target_id;
        let key = NodeID::from_vec(key_bytes);
        let header = req.header.unwrap();
        self.insert_update(header.my_id,&header.pub_key,remote_addr);
        self.node.store_value(key, req.value);
        let timestamp = Utc::now().timestamp();
        let reply = StoreRepl {
            cookie: req.cookie,
            header: Some( Header { 
                my_id: self.node.uid.as_bytes().to_owned(),
                pub_key: self.node.get_pubkey(),
                nonce: self.node.get_nonce(),
                timestamp,
                signature : Signer::sign_weak_header_rep(timestamp,&header.pub_key,&self.node.address, &header.signature) 
            }),
        };

        Ok(Response::new(reply))
    }

    async fn find_value(&self, request: Request<FValueReq>) -> Result<Response<FValueRepl>,Status>{
        let remote_addr = request.remote_addr().unwrap();
        let req = request.into_inner();
        let key_bytes = req.target_id;
        let lookup_key = NodeID::from_vec(key_bytes);
        let header = req.header.unwrap();
        self.insert_update(header.my_id,&header.pub_key,remote_addr);
        let has_value : HasValue;
        match self.node.retrieve(lookup_key) {
            Some(val) => has_value = Value(val.as_bytes().to_owned()),
            None => has_value = HNode( Kclosest { 
                                        node : self.lookup(lookup_key),
                                }),
        };
        let mut databuf = Vec::new();
        has_value.encode(&mut databuf);
        let timestamp = Utc::now().timestamp();
        let reply = FValueRepl {
            cookie: req.cookie,
            header: Some(Header { 
                my_id: self.node.uid.as_bytes().to_owned(),
                pub_key: self.node.get_pubkey(),
                nonce: self.node.get_nonce(),
                timestamp,
                signature : Signer::sign_strong_header_rep(timestamp,&header.pub_key, &self.node.address,databuf, &header.signature),
            }),
            has_value: Some(has_value),
        };

        Ok(Response::new(reply))
    }

    async fn find_node(&self, request: Request<FNodeReq>) -> Result<Response<FNodeRepl>,Status>{
        let remote_addr = request.remote_addr().unwrap();
        let req = request.into_inner();
        let header = req.header.unwrap();
      
        let key_bytes = req.target_id;
        if let Ok(()) = Signer::validate_strong_req(self.node.get_validator(),&header,&self.node.address,&key_bytes) {
            let lookup_key = NodeID::from_vec(key_bytes);
            let k = Kclosest {
                node : self.lookup(lookup_key),
            };
            self.insert_update(header.my_id,&header.pub_key,remote_addr);
            let mut databuf = Vec::new();
            let _enc = k.encode(&mut databuf).unwrap();
            let timestamp = Utc::now().timestamp();
            let reply = FNodeRepl {
                cookie: req.cookie,
                header: Some( Header { 
                    my_id: self.node.uid.as_bytes().to_owned(),
                    pub_key: self.node.get_pubkey(),
                    nonce: self.node.get_nonce(),
                    timestamp,
                    signature : Signer::sign_strong_header_rep(timestamp,&header.pub_key, &self.node.address,databuf, &header.signature),
                }),
                nodes: Some(k),
            };
            return Ok(Response::new(reply));
        }
        Err(Status::new(Code::InvalidArgument, "Invalid message"))
        
    }
}
