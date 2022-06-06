use std::sync::Arc;

use prost::Message;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Code};
use crate::{p2p::util::{gen_cookie, grpc_block}};

use super::{kad::KadNode, 
    key::NodeID, 
    node::{Contact}, 
    signatures::Signer, 
    kademlia::{
        kademlia_server::{Kademlia, KademliaServer}, 
        PingM, Kcontact, StoreReq, StoreRepl, FValueReq, FValueRepl,
        f_value_repl::{HasValue::{Auction,Node as HNode}, HasValue},
        Kclosest, Header, FNodeReq, FNodeRepl, Auctions, BroadcastReq, Empty, Gblock, kademlia_client::KademliaClient}, util::{to_gossip, to_auction_data_vec, encode_fvalue, encode_store, format_address, to_data, to_block, build_brequest}};

#[derive(Debug)]
pub struct KademliaProtocol{
    pub node: Arc<KadNode>,
}

impl KademliaProtocol {
    pub fn new(node: Arc<KadNode>) -> KademliaProtocol {
        KademliaProtocol {
            node,
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

    fn insert_update(&self,id: Vec<u8>,pub_key: &[u8], remote_addr: String) {
        let con = Contact::new(NodeID::from_vec(id), remote_addr, pub_key.to_vec());
        self.node.insert(con);
    }

}

#[tonic::async_trait]
impl Kademlia for KademliaProtocol {
   async fn ping(&self, request: Request<PingM>) -> Result<Response<PingM>,Status>{
        if let Some(sender_addr) = request.remote_addr() {
            println!("Hello from the server side {:?}",sender_addr);
        }
        println!("validating {:?}", &request);
        let remote_addr = request.remote_addr().unwrap();
        let req = request.into_inner();
        let header = req.header.unwrap();
        
        if let Ok(req_hash) = Signer::validate_weak_req(self.node.get_validator(),&header,&remote_addr.to_string()) {
            println!("validated ping{:?}", remote_addr);

            self.insert_update(header.my_id,&header.pub_key,header.address);
            let timestamp = gen_cookie();
            let reply = PingM {
                header: Some( Header {
                    my_id: self.node.uid.as_bytes().to_owned(),
                    address : self.node.address.to_owned(),
                    pub_key: self.node.get_pubkey(),
                    nonce: self.node.get_nonce(),
                    timestamp,
                    signature: Signer::sign_weak_header_rep(timestamp,&header.pub_key, &self.node.address, &req_hash),
                }),
            };
            println!("Sending reply: {:?}", reply);
            return Ok(Response::new(reply));
        }
        Err(Status::new(Code::InvalidArgument, "Invalid message"))
    }

    async fn store(&self, request: Request<StoreReq>) -> Result<Response<StoreRepl>,Status>{
        let remote_addr = request.remote_addr().unwrap();
        let req = request.into_inner();
        let header = req.header.unwrap();
        let key =NodeID::from_vec(req.target_id);
        let value = req.value.unwrap();
        let databuf = encode_store(&value,key);
        if let Ok(req_hash) = Signer::validate_strong_req(self.node.get_validator(),&header,&remote_addr.to_string(),&databuf) {
            println!("validated store from {:?}", remote_addr);
            self.insert_update(header.my_id,&header.pub_key,header.address);
            let value = to_gossip(&value);
            let timestamp = self.node.compare(header.timestamp);
            if timestamp == header.timestamp + 1 {
            let _ = self.node.store_value(key, value);
            }
            let reply = StoreRepl {
                header: Some( Header { 
                    my_id: self.node.uid.as_bytes().to_owned(),
                    address : self.node.address.to_owned(),
                    pub_key: self.node.get_pubkey(),
                    nonce: self.node.get_nonce(),
                    timestamp,
                    signature : Signer::sign_weak_header_rep(timestamp,&header.pub_key,&self.node.address, &req_hash) 
                }),
            };

            return Ok(Response::new(reply));
    }
    Err(Status::new(Code::InvalidArgument, "Invalid message"))

    }

    async fn find_value(&self, request: Request<FValueReq>) -> Result<Response<FValueRepl>,Status>{
        let remote_addr = request.remote_addr().unwrap();
        let req = request.into_inner();
        let key_bytes = req.target_id;
        let header = req.header.unwrap();
        if let Ok(req_hash) = Signer::validate_strong_req(self.node.get_validator(),&header,&remote_addr.to_string(),&key_bytes) {
            println!("validated find value from {:?}", remote_addr);
            self.insert_update(header.my_id,&header.pub_key,header.address);
            let lookup_key = NodeID::from_vec(key_bytes);
            let has_value : HasValue;
            match self.node.retrieve(lookup_key) {
                Some(val) => has_value = Auction(Auctions { list: to_auction_data_vec(val)} ),
                None => has_value = HNode( Kclosest { 
                                            node : self.lookup(lookup_key),
                                    }),
            };
            let databuf = encode_fvalue(&has_value, lookup_key);
            let timestamp = self.node.compare(header.timestamp);
            let reply = FValueRepl {
                header: Some(Header { 
                    my_id: self.node.uid.as_bytes().to_owned(),
                    address : self.node.address.to_owned(),
                    pub_key: self.node.get_pubkey(),
                    nonce: self.node.get_nonce(),
                    timestamp,
                    signature : Signer::sign_strong_header_rep(timestamp,&header.pub_key,&self.node.address,databuf, &req_hash),
                }),
                has_value: Some(has_value),
            };

            return Ok(Response::new(reply));
        }
    Err(Status::new(Code::InvalidArgument, "Invalid message"))

    }

    async fn find_node(&self, request: Request<FNodeReq>) -> Result<Response<FNodeRepl>,Status>{
        let remote_addr = request.remote_addr().unwrap();
        let req = request.into_inner();
        let header = req.header.unwrap();
        let key_bytes = req.target_id;
        if let Ok(req_hash) = Signer::validate_weak_req(self.node.get_validator(),&header,&remote_addr.to_string()) {
            println!("validated find node {:?}", remote_addr);

            let lookup_key = NodeID::from_vec(key_bytes);
            let k = Kclosest {
                node : self.lookup(lookup_key),
            };
            self.insert_update(header.my_id,&header.pub_key,header.address);
            let mut databuf = Vec::new();
            let _enc = k.encode(&mut databuf).unwrap();
            let timestamp = self.node.compare(header.timestamp);
            let reply = FNodeRepl {
                header: Some( Header { 
                    my_id: self.node.uid.as_bytes().to_owned(),
                    address : self.node.address.to_owned(),
                    pub_key: self.node.get_pubkey(),
                    nonce: self.node.get_nonce(),
                    timestamp,
                    signature : Signer::sign_strong_header_rep(timestamp,&header.pub_key, &self.node.address,databuf, &req_hash),
                }),
                nodes: Some(k),
            };
            return Ok(Response::new(reply));
        }
        Err(Status::new(Code::InvalidArgument, "Invalid message"))
        
    }

    async fn broadcast(&self, request: Request<BroadcastReq>) -> Result<Response<Empty>,Status> {
        println!("broadcast{:?}", request.remote_addr());
        let req = request.into_inner();
        let timestamp = self.node.compare_broadcast(req.timestamp);
        if timestamp == req.timestamp + 1 {
            let data = &req.rdata.unwrap();
            match data {
                super::kademlia::broadcast_req::Rdata::Block(b) => {
                    self.node.store_block(to_block(b.clone()));
                    if let Err(e) = self.node.validate_blocks() {
                        println!("{}", e);
                    }
                },
                super::kademlia::broadcast_req::Rdata::Transaction(t) => {
                    self.node.store_transaction(to_data(t.clone()));
                    let _ = self.node.mine_and_broadcast().await;
                },
            }
            let my_closest = self.lookup(self.node.uid);
            for close in my_closest {
                let connection = KademliaClient::connect(format_address(close.address)).await;
                match connection {
                    Ok(mut chan) => {
                        let _ = chan.broadcast(build_brequest(&req.timestamp,&data)).await;
                    },
                    Err(_) => (),
                }
            }
            
        }

        Ok(Response::new(Empty{}))
        
    }

    type req_chainStream =  ReceiverStream<Result<Gblock, Status>>;

    async fn req_chain(&self, request: Request<PingM>) -> Result<Response<Self::req_chainStream>, Status> {
        let remote_addr = request.remote_addr().unwrap();
        let req = request.into_inner();
        let header = req.header.unwrap();
        
        if let Ok(_) = Signer::validate_weak_req(self.node.get_validator(),&header,&remote_addr.to_string()) {
            println!("validated chain{:?}", remote_addr);
            let _timestamp = self.node.compare(header.timestamp);
            let (tx, rx) = mpsc::channel(4);
            let chain= self.node.get_chain();
            tokio::spawn(async move {
                for block in chain.blocks.iter() {
                        tx.send(Ok(grpc_block(block.clone()))).await.unwrap();
                }
            });

            Ok(Response::new(ReceiverStream::new(rx)))
        } 
        else {
            Err(Status::new(Code::InvalidArgument, "Invalid message"))
        }
    }
}
