use super::{node::{Contact, LastSeen}, key::Key};
use rand::Rng;
use kademlia::{
    kademlia_client::KademliaClient,
    PingM,Kcontact
};
use tonic::{Request};

mod kademlia {
    tonic::include_proto!("kadproto");
}

pub fn format_address(ip: String, port: u16) -> String {
    ("http://".to_owned() + &ip + ":" + &port.to_string()).to_owned()
}

pub fn format_kcontact(contact: Contact) -> Kcontact {
    Kcontact {
        uid : contact.uid.as_bytes().to_owned(),
        ip: contact.ip.clone(),
        port: contact.port.clone() as i32,
    }
}

pub fn gen_cookie() -> String {
    let mut rng = rand::thread_rng();
    let cookie: usize = rng.gen();
    cookie.to_string()
}

pub async fn send_ping(my_key: Key,contact: Contact) -> bool {
    let addr = format_address(contact.ip,contact.port);
    let mut client = KademliaClient::connect(addr).await.unwrap();  
    let request = PingM {
            cookie: gen_cookie(),
            id : my_key.as_bytes().to_owned(),
        };

    match client.ping(request).await {
        Ok(_) => true,
        Err(_) => false,
    }
}
