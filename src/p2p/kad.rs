use std::collections::HashMap;

use parking_lot::RwLock;
use to_binary::BinaryString;
use chrono::prelude::*;

use super::{
    key::Key,
    rtable::Rtable,
    node::{Node, Contact,LastSeen},
};

#[derive(Debug)]
pub struct KadNode {
    pub uid: Key,
    pub ip: String,
    pub port: u16,
    pub join_date: DateTime<Utc>,
    pub rtable: RwLock<Rtable>,
    pub data_store: RwLock<HashMap<Key,String>>,
}

impl KadNode {
    pub fn new(ip: String, port: u16) -> KadNode {
       KadNode {
            uid: Key::new(ip.clone() + &port.to_string()),
            ip: ip,
            port: port,
            rtable: RwLock::new(Rtable::new()),
            join_date: Utc::now(),
            data_store: RwLock::new(HashMap::new()),
        }
    }

    pub fn lookup(&self,id: Key) -> Vec<Box<Contact>> {
        self.rtable.read().lookup(id)
    }

    pub fn as_contact(&self) -> Contact {
        Contact {
            uid: self.uid.clone(),
            ip: self.ip.clone(),
            port: self.port.clone(),
            last_seen: LastSeen::Never,
        }
    }

    pub fn insert(&self,contact:Contact) {
        self.rtable.write().insert(contact, self.uid)
    }

    pub fn print_rtable(&self) {
        println!("{:?}",self.rtable.try_read().unwrap().head);
    }

    pub fn store_value(&self, key: Key, value: String) -> Option<String> {
        self.data_store.write().insert(key, value)
    }

    pub fn retrieve(&self, key: Key) -> Option<String> {
        if let Some(value) = self.data_store.read().get(&key) {
            Some(value.clone())
        } else {
            None
        }
    }
}
