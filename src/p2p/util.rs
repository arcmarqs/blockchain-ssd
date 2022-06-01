use chrono::Utc;
use rand::Rng;

use super::{key::NodeValidator, node::Contact, signatures::Signer, kademlia::{kademlia_client::KademliaClient, PingM, Kcontact, Header}};


pub fn format_address(ip: String, port: u16) -> String {
    ("http://".to_owned() + &ip + ":" + &port.to_string()).to_owned()
}

pub fn format_kcontact(contact: Contact) -> Kcontact {
    Kcontact {
        uid : contact.uid.as_bytes().to_owned(),
        address: contact.address.clone(),
        pub_key: contact.get_pubkey().to_vec(),
    }
}

pub fn gen_cookie() -> String {
    let mut rng = rand::thread_rng();
    let cookie: usize = rng.gen();
    cookie.to_string()
}

pub async fn send_ping(validator: &NodeValidator, contact: Contact) -> bool {
    let mut client = KademliaClient::connect(contact.address.clone()).await.unwrap();
    let timestamp = Utc::now().timestamp();
    let request = PingM {
            cookie: gen_cookie(),
            header: Some( Header {
                my_id: validator.get_nodeid().as_bytes().to_owned(),
                pub_key: validator.get_pubkey(),
                nonce: validator.get_nonce(),
                timestamp: timestamp,
                signature: Signer::sign_weak_header_req(timestamp,&validator.get_pubkey(),&contact.address),
            }),
        };

    match client.ping(request).await {
        Ok(_) => true,
        Err(_) => false,
    }
}
