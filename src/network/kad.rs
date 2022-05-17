use crate::network::{rtable::Rtable,node::{Contact}};
use crate::key::{Key};
use to_binary::BinaryString;


#[derive(Debug,Clone)]
pub struct KadNode {
    uid: Key,
    ip: String,
    port: u16,
    rtable: Rtable,
}

impl KadNode {
    pub fn new(ip: String, port: u16) -> KadNode {
        let key = Key::new(ip.clone() + &port.to_string());
        let origin = Contact::new(key,ip.clone(),port);
        let mut r = Rtable::new();
        r.init(origin);
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

    pub fn as_contact(&self) -> Contact {
        Contact {
            uid: self.uid.clone(),
            ip: self.ip.clone(),
            port: self.port.clone(),
        }
    }
}
