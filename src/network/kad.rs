use crate::network::rtable::{Rtable};
use crate::key::{Key};

#[derive(Debug,Clone)]
pub struct KadNode {
    uid: Key,
    ip: String,
    port: u8,
    rtable: Rtable,
}

impl KadNode {
    pub fn bootstrap() {
        let ip = String::from("0.0.0.0");
        let port = 5050;
        let uid = Key::new(ip + &port.to_string());

    }
}
