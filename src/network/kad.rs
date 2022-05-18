use crate::network::{rtable::Rtable,node::{Contact,Node}};
use crate::key::{Key};
use to_binary::BinaryString;


#[derive(Debug,Clone)]
pub struct KadNode {
    pub uid: Key,
    ip: String,
    port: u16,
    rtable: Rtable,
}

impl KadNode {
    pub fn new(ip: String, port: u16) -> KadNode {
        let key = Key::new(ip.clone() + &port.to_string());
        let origin = Contact::new(key,ip.clone(),port);
        let mut r = Rtable::new();
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

    pub fn lookup(&self,id: Key) -> &Node {
        self.rtable.lookup(id)
    }

    pub fn as_contact(&self) -> Contact {
        Contact {
            uid: self.uid.clone(),
            ip: self.ip.clone(),
            port: self.port.clone(),
        }
    }

    pub fn insert(&mut self,contact:Contact) {
        self.rtable.insert(contact,self.uid)
    }

    pub fn print_rtable(&self) {
        println!("{:?}",self.rtable)
    }
}
