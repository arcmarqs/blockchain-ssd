use std::{collections::HashMap, sync::{atomic::AtomicU64, Arc}};

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use tonic::Request;
use std::sync::atomic::Ordering::{SeqCst,Acquire};
use crate::{auctions::auction::AuctionGossip, ledger::{block::{Chain, Block, Data}, miner::Miner}};

use super::{
    key::{NodeValidator, NodeID}, 
    rtable::Rtable, node::Contact, 
    kademlia::{kademlia_client::KademliaClient, Header, StoreReq, BroadcastReq, PingM}, 
    util::{format_address, encode_store, to_auction_data, grpc_block, to_block}, 
    signatures::Signer
};

#[derive(Debug)]
pub struct KadNode {
    pub uid: NodeID,
    pub address: String,
    pub join_date: DateTime<Utc>,
    timestamp: Arc<AtomicU64>,
    last_broadcast: AtomicU64,
    validator: NodeValidator,
    rtable: RwLock<Rtable>,
    data_store: RwLock<HashMap<NodeID,Vec<AuctionGossip>>>,
    miner: Miner
}

impl KadNode {
    pub fn new(addr: String) -> KadNode {
        let valid = NodeValidator::new();
        let date = Utc::now();
       KadNode {
            uid: valid.get_nodeid(),
            address: addr,
            rtable: RwLock::new(Rtable::new()),
            join_date: date,
            timestamp: Arc::new(AtomicU64::new(0)),
            last_broadcast: AtomicU64::new(0),
            data_store: RwLock::new(HashMap::new()),
            validator : valid,
            miner: Miner::new(), 
        }
    }

    pub fn lookup(&self,id: NodeID) -> Vec<Box<Contact>> {
        self.rtable.read().lookup(id)
    }
/* UNUSED 
    pub fn as_contact(&self) -> Contact {
        Contact::new(
            self.uid.clone(),
            self.address.clone(),
            self.validator.get_pubkey(),
        )
    }
*/

    pub fn insert(&self,contact:Contact) {
        self.rtable.write().insert(&self.address,contact, &self.validator)
    }

    pub fn print_rtable(&self) {
        println!("{:?}",self.rtable.try_read().unwrap().head);
    }

    pub fn print_store(&self) {
        println!("{:?}",self.data_store.try_read().unwrap());
    }

    pub fn print_blockchain(&self) {
        self.miner.print_blockchain()
    }

    pub fn store_value(&self, key: NodeID, value: AuctionGossip) -> Result<(), &'static str> {
        let mut keys = self.get_store_keys(&value);
        if !keys.contains(&key) {
            keys.push(key);
        }

        let mut lock = self.data_store.write();
        for k in keys.iter() {
            match lock.get_mut(k) {
                Some(vec) => {
                    let mut id = 0;
                        for v in vec.iter() {
                            if v.get_price() > value.get_price() {
                                return Err("Invalid bid")
                            }
                            if v == &value {
                            vec.remove(id);
                            break;
                            }
                            id +=1;
                        }
                        vec.push(value.clone());

                    }
                None => {
                    lock.insert(*k, vec![value.clone()]);
                }
            }
        }
        let _ = self.publish_auction(keys, value);
        Ok(())
     }

    pub fn retrieve(&self, key: NodeID) -> Option<Vec<AuctionGossip>> {
        if let Some(value) = self.data_store.read().get(&key) {
            Some(value.clone())
        } else {
            None
        }
    }

    pub fn get_nonce(&self) -> u64 {
        self.validator.get_nonce()
    }

    pub fn get_pubkey(&self) -> Vec<u8> {
        self.validator.get_pubkey()
    }

    pub fn get_validator(&self) -> &NodeValidator {
        &self.validator
    }

    pub fn get_uid(&self) -> NodeID {
        self.uid.clone()
    }

    pub fn get_timestamp(&self) -> Arc<AtomicU64> {
        self.timestamp.clone()
    }

    pub fn get_chain(&self) -> Chain {
        self.miner.get_chain()
    }
    
    pub fn increment(&self) -> u64 {
        self.timestamp.fetch_add(1, SeqCst)
    }

    
    pub fn increment_broadcast(&self) -> u64 {
        self.last_broadcast.fetch_add(1, SeqCst)
    }

    pub fn compare_broadcast(&self, other: u64) -> u64 {
        loop {
            let cur = self.last_broadcast.load(SeqCst);
            if cur >= other {
               return cur;
            } else {
                match self.last_broadcast.compare_exchange(cur, other+1, SeqCst, Acquire) {
                    Ok(value) => return value,
                    Err(value) => {
                        if value >= other {
                           return value;
                        }
                    },
                }
            }
        }
    }

    // syncronizes timestamp
    pub fn compare(&self, other: u64) -> u64 {
        loop {
            let cur = self.timestamp.load(SeqCst);
            if cur >= other {
               return cur;
            } else {
                match self.timestamp.compare_exchange(cur, other+1, SeqCst, Acquire) {
                    Ok(value) => return value,
                    Err(value) => {
                        if value >= other {
                           return value;
                        }
                    },
                }
            }
        }
    }

    pub fn store_transaction(&self, t: Data) {
        self.miner.store_transaction(t)
    }

    pub fn store_block(&self,block: Block) {
        self.miner.store_block(block)
    }

    fn get_store_keys(&self,value: &AuctionGossip) -> Vec<NodeID> {
        let mut keys : Vec<NodeID> = Vec::new();
        let lock = self.data_store.read();

        for (id,vals) in lock.iter() {
            if vals.contains(value) {
                keys.push(id.clone());
            }
        }

        keys
    }
    
    async fn publish_auction(&self, keys: Vec<NodeID>, value:AuctionGossip) {
        for k in keys.iter() {
            if k == &self.uid {
                continue
            }
            let contacts = self.lookup(k.clone());

            for contact in contacts {
                let _ = self.send_publish(k.clone(), value.clone(), *contact).await;
            }
        }
    }

    async fn send_publish(&self, target_key: NodeID, value:AuctionGossip, contact: Contact) -> Result<(),&'static str>  {
        let address = format_address(contact.address.clone());
        let mut client = KademliaClient::connect(address.clone()).await.unwrap();  
        let formated_value = to_auction_data(value);
        let timestamp =  self.increment();
        let databuf: Vec<u8> = encode_store(&formated_value,target_key);
        let (hash,request_signature) = Signer::sign_strong_header_req(timestamp,contact.get_pubkey(),&self.address,&databuf);
        let request = StoreReq {
                header: Some( Header {
                    my_id: self.uid.as_bytes().to_owned(),
                    address : self.address.to_owned(),
                    pub_key: self.get_pubkey(),
                    nonce: self.get_nonce(),
                    timestamp,
                    signature: request_signature.clone() ,
                }),
                target_id: target_key.as_bytes().to_owned(),
                value: Some(formated_value),
            };
        let response =  client.store(request).await;

        match response {
            Ok(res) => {
                let res = res.into_inner();
                let header = res.header.unwrap();
                if let Ok(()) = Signer::validate_weak_rep(self.get_validator(),&header,&contact.address,&hash) {
                    return Ok(());
                }
                Err("Failed to verify signature")
             },
            Err(_) => Err("Failed to unwrap response"),
        }
    }

    pub async fn mine_and_broadcast(&self) {
        let block = self.miner.mine();
        let my_closest = self.lookup(self.uid);
        let timestamp = self.increment_broadcast();
        let data = grpc_block(block.clone());
        

        for contact in my_closest {
            let connection = KademliaClient::connect(format_address(contact.address)).await; 

            match connection {
                Ok(mut channel) => {
                    let broadcast_message = Request::new(BroadcastReq { 
                        timestamp, 
                        rdata:  Some(super::kademlia::broadcast_req::Rdata::Block(data.clone())),
                    });
                    let _ = channel.broadcast(broadcast_message);
                },
                Err(_) => continue,
            }
        }
    }

    pub async fn request_chain(&self)  -> Result<(), Box<dyn std::error::Error>> {
        let timestamp =  self.increment();
        let closest = self.lookup(self.uid);
        let mut chains:  Vec<Vec<Block>> = Vec::new();
        for close in closest {
            let client = KademliaClient::connect(format_address(close.address.clone())).await;
            let mut chain : Vec<Block> = Vec::new();
            match client {
                Ok(mut channel) => {
                    let (_,request_signature) = Signer::sign_weak_header_req(timestamp,close.get_pubkey(),&self.address);
                    let request = PingM {
                        header: Some( Header {
                            my_id: self.validator.get_nodeid().as_bytes().to_owned(),
                            address : self.address.to_owned(),
                            pub_key: self.validator.get_pubkey(),
                            nonce: self.validator.get_nonce(),
                            timestamp: timestamp.clone(),
                            signature: request_signature.clone(),
                        }),
                    };
        
                    let mut stream = channel
                        .req_chain(Request::new(request))
                        .await?
                        .into_inner();
                  
                    
                    while let Some(block) = stream.message().await? {
                        chain.push(to_block(block));
                    }
                },
                Err(_) => continue,
            }

            if !chain.is_empty() {
                chains.push(chain);
            }
        }

        self.miner.choose_chain(chains);
        Ok(())
    }

    pub fn validate_blocks(&self) -> Result<(), &'static str>{
        self.miner.validate_blocks()
    }
}
