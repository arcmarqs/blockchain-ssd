use std::{collections::HashMap, sync::{atomic::AtomicU64, Arc}};

use chrono::{DateTime, Utc};
use parking_lot::{RwLock, Mutex};
use std::sync::atomic::Ordering::{SeqCst,Acquire};
use crate::auctions::auction::AuctionGossip;

use super::{key::{NodeValidator, NodeID}, rtable::Rtable, node::Contact};

#[derive(Debug)]
pub struct KadNode {
    pub uid: NodeID,
    pub address: String,
    pub join_date: DateTime<Utc>,
    timestamp: Arc<AtomicU64>,
    validator: NodeValidator,
    rtable: RwLock<Rtable>,
    data_store: RwLock<HashMap<NodeID,Vec<AuctionGossip>>>,
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
            data_store: RwLock::new(HashMap::new()),
            validator : valid,
        }
    }

    pub fn lookup(&self,id: NodeID) -> Vec<Box<Contact>> {
        self.rtable.read().lookup(id)
    }

    pub fn as_contact(&self) -> Contact {
        Contact::new(
            self.uid.clone(),
            self.address.clone(),
            self.validator.get_pubkey(),
        )
    }

    pub fn insert(&self,contact:Contact) {
        self.rtable.write().insert(&self.address,contact, &self.validator)
    }

    pub fn print_rtable(&self) {
        println!("{:?}",self.rtable.try_read().unwrap().head);
    }

    pub fn store_value(&self, key: NodeID, value: AuctionGossip) -> Result<(), &'static str> {
        let mut lock = self.data_store.write();

        match lock.get_mut(&key) {
            Some(vec) => {
                let mut id = 0;
                    for v in vec.clone().iter() {
                        if v == &value {
                           vec.remove(id);
                        }
                        id +=1;
                    }
                    vec.push(value);
                    Ok(())
                }
            None => {
                match lock.insert(key, vec![value]){
                    Some(_) => Ok(()),
                    None => Err("failed to insert key"),
                }
            },
        }
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
   
    pub fn increment(&self) -> u64 {
        self.timestamp.fetch_add(1, SeqCst)
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
}
