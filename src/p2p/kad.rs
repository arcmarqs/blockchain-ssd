use to_binary::BinaryString;

use super::{
    key::Key,
    rtable::Rtable,
    node::{Node, Contact},
};

#[derive(Debug,Clone,Default)]
pub struct KadNode {
    pub uid: Key,
    pub ip: String,
    pub port: u16,
    rtable: Rtable,
}

impl KadNode {
    pub fn new(ip: String, port: u16) -> KadNode {
        let key = Key::new(ip.clone() + &port.to_string());
        let origin = Contact::new(key,ip.clone(),port);
        let r = Rtable::new();
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

    pub fn lookup(&self,id: Key) -> Vec<Box<Contact>> {
        self.rtable.lookup(id)
    }

    pub fn as_contact(&self) -> Contact {
        Contact {
            uid: self.uid,
            ip: self.ip.clone(),
            port: self.port,
        }
    }

    pub fn insert(&mut self,contact:Contact) {
        self.rtable.insert(contact,self.uid)
    }

    pub fn print_rtable(&self) {
        println!("{:?}",self.rtable)
    }
}
