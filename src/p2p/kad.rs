use std::collections::HashMap;

use chrono::{DateTime, Utc};
use parking_lot::RwLock;

use super::{key::{NodeValidator, NodeID}, rtable::Rtable, node::Contact};

#[derive(Debug)]
pub struct KadNode {
    pub uid: NodeID,
    pub address: String,
    pub join_date: DateTime<Utc>,
    timestamp: i64,
    validator: NodeValidator,
    rtable: RwLock<Rtable>,
    data_store: RwLock<HashMap<NodeID,String>>,
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
            timestamp: date.timestamp(),
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

    pub fn store_value(&self, key: NodeID, value: String) -> Option<String> {
        self.data_store.write().insert(key, value)
    }

    pub fn retrieve(&self, key: NodeID) -> Option<String> {
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
}
