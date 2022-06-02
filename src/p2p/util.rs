use chrono::Utc;
use rand::Rng;

use super::{
    kademlia::{kademlia_client::KademliaClient, Header, Kcontact},
    key::NodeValidator,
    node::Contact,
    signatures::Signer,
};

pub fn format_address(address: String) -> String {
    "http://".to_owned() + &address
}

pub fn format_kcontact(contact: Contact) -> Kcontact {
    Kcontact {
        uid: contact.uid.as_bytes().to_owned(),
        address: contact.address.clone(),
        pub_key: contact.get_pubkey().to_vec(),
    }
}

pub fn gen_cookie() -> String {
    let mut rng = rand::thread_rng();
    let cookie: usize = rng.gen();
    cookie.to_string()
}
