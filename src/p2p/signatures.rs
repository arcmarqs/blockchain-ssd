use openssl::sha::Sha256;

use super::{
    kademlia::Header,
    key::{encrypt_message, verify_puzzle, NodeID, NodeValidator},
};

pub struct Signer {}
impl Signer {
    pub fn sign_strong_header_req(timestamp: i64, pub_key: &[u8], address: &str, data: Vec<u8>) -> (Vec<u8>,Vec<u8>) {
        let mut hasher = Sha256::new();
        hasher.update(&timestamp.to_be_bytes());
        hasher.update(&address.as_bytes());
        hasher.update(&data);
        let signature = hasher.finish().to_vec();
        (signature.clone(),encrypt_message(pub_key, &signature))
    }

    pub fn sign_strong_header_rep(timestamp: i64, pub_key: &[u8], address: &str, data: Vec<u8>, rep_signature: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(&timestamp.to_be_bytes());
        hasher.update(&address.as_bytes());
        hasher.update(&data);
        hasher.update(rep_signature);
        encrypt_message(pub_key, &hasher.finish())
    }

    pub fn sign_weak_header_req(timestamp: i64, pub_key: &[u8], address: &str) -> (Vec<u8>, Vec<u8>) {
        println!("HASHER GOES OMNOMNOM ON: {:?} {:?}",  timestamp, address);
        let mut hasher = Sha256::new();
        hasher.update(&timestamp.to_be_bytes());
        hasher.update(address.as_bytes());
        let signature = hasher.finish().to_vec();
        (signature.clone(),encrypt_message(pub_key, &signature))
    }

    pub fn sign_weak_header_rep(timestamp: i64,pub_key: &[u8],address: &str,rep_signature: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(&timestamp.to_be_bytes());
        hasher.update(address.as_bytes());
        hasher.update(rep_signature);

        encrypt_message(pub_key, &hasher.finish())
    }

    pub fn validate_weak_req(validator: &NodeValidator,header: &Header, address: &str) -> Result<Vec<u8>, &'static str> {
        let node_id = NodeID::from_vec(header.my_id.clone());
        let nonce = header.nonce;
        let mut signature = validator.decrypt(&header.signature);
        signature.truncate(32);
        let timestamp = header.timestamp;

        if verify_puzzle(node_id, nonce) {
            println!("HASHER GOES OMNOMNOM ON: {:?} {:?}",  timestamp, address);
            let mut hasher = Sha256::new();
            hasher.update(&timestamp.to_be_bytes());
            hasher.update(address.as_bytes());
            let sign = hasher.finish().to_vec();
            if signature == sign {
                println!("inside");
                return Ok(sign);
            }
        }

        Err("invalid message")
    }

    pub fn validate_weak_rep(validator: &NodeValidator, header: &Header, address: &str,req_signature: &[u8]) -> Result<(), &'static str> {
        let node_id = NodeID::from_vec(header.my_id.clone());
        let nonce = header.nonce;
        let mut signature = validator.decrypt(&header.signature);
        signature.truncate(32);
        let timestamp = header.timestamp;

        if verify_puzzle(node_id, nonce) {
            let mut hasher = Sha256::new();
            hasher.update(&timestamp.to_be_bytes());
            hasher.update(address.as_bytes());
            hasher.update(req_signature);
            let sign = hasher.finish().to_vec();
            if sign == signature {
                return Ok(());
            }
        }

        Err("invalid message")
    }

    pub fn validate_strong_rep(validator: &NodeValidator,header: &Header,address: &str,data: &[u8],req_signature: &[u8]) -> Result<(), &'static str> {
        let node_id = NodeID::from_vec(header.my_id.clone());
        let nonce = header.nonce;
        let mut signature = validator.decrypt(&header.signature);
        signature.truncate(32);
        let timestamp = header.timestamp;

        if verify_puzzle(node_id, nonce) {
            let mut hasher = Sha256::new();
            hasher.update(&timestamp.to_be_bytes());
            hasher.update(address.as_bytes());
            hasher.update(data);
            hasher.update(req_signature);
            let sign = hasher.finish().to_vec();
            if sign == signature {
                return Ok(());
            }
        }

        Err("invalid message")
    }

    pub fn validate_strong_req(validator: &NodeValidator, header: &Header, address: &str, data: &[u8]) -> Result<Vec<u8>, &'static str> {
        let node_id = NodeID::from_vec(header.my_id.clone());
        let nonce = header.nonce;
        let mut signature = validator.decrypt(&header.signature);
        signature.truncate(32);
        let timestamp = header.timestamp;

        if verify_puzzle(node_id, nonce) {
            let mut hasher = Sha256::new();
            hasher.update(&timestamp.to_be_bytes());
            hasher.update(address.as_bytes());
            hasher.update(data);

            let sign = hasher.finish().to_vec();
            if sign == signature {
                return Ok(sign);
            }
        }

        Err("invalid message")
    }
}
