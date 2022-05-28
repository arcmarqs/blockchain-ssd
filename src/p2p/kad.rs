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
}

impl KadNode {
    pub fn new(ip: String, port: u16) -> KadNode {
        let key = Key::new(ip.clone() + &port.to_string());
        let origin = Contact::new(key,ip.clone(),port);
        let r = RwLock::new(Rtable::new());
        KadNode {
            uid: key,
            ip: ip,
            port: port,
            rtable: r,
            join_date: Utc::now(),
        }
    }
    pub fn bootstrap() {
        let ip = String::from("0.0.0.0");
        let port:u16 = 5050;
        let uid = Key::new(ip + &port.to_string());
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

    pub fn insert(&mut self,contact:Contact) {
        self.rtable.write().insert(contact, self.uid)
    }

    pub fn print_rtable(&self) {
        println!("{:?}",self.rtable.try_read().unwrap().head);
    }
}
