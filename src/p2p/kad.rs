use to_binary::BinaryString;
use tokio::sync::RwLock;

use super::{
    key::Key,
    rtable::Rtable,
    node::{Node, Contact},
};

#[derive(Debug,Default)]
pub struct KadNode {
    pub uid: Key,
    pub ip: String,
    pub port: u16,
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
            rtable: r
        }
    }
    pub fn bootstrap() {
        let ip = String::from("0.0.0.0");
        let port:u16 = 5050;
        let uid = Key::new(ip + &port.to_string());
    }

    pub async fn lookup(&self,id: Key) -> Vec<Box<Contact>> {
        self.rtable.read().await.lookup(id)
    }

    pub fn as_contact(&self) -> Contact {
        Contact {
            uid: self.uid.clone(),
            ip: self.ip.clone(),
            port: self.port.clone(),
        }
    }

    pub async fn insert(&mut self,contact:Contact) {
        self.rtable.write().await.insert(contact,self.uid)
    }

    pub async fn print_rtable(&self) {
        println!("{:?}",self.rtable.try_read().unwrap().head);
    }
}
