use chrono::Utc;
use primitive_types::H256;
use prost::Message;
use rand::Rng;

use crate::auctions::auction::{AuctionGossip, AuctionState};

use super::{
    kademlia::{kademlia_client::KademliaClient, Header, Kcontact, AuctionData, f_value_repl::HasValue},
    key::{NodeValidator, NodeID},
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

pub fn to_gossip(auction: &AuctionData) -> AuctionGossip {
    let id = |bytes: &Vec<u8>| {
        H256::from_slice(bytes.as_slice())
    }; 

    let state = |a: bool| {
        match a {
            true => AuctionState::ONGOING,
            false => AuctionState::FINISHED,
        }
    };

    AuctionGossip::new(
        id(&auction.auction_id), 
        auction.title.clone(),
        auction.price,
        state(auction.status),
        NodeID::from_vec(auction.seller.clone()))
}

pub fn to_gossip_vec(auctions: Vec<AuctionData>) -> Vec<AuctionGossip> {
    auctions.iter().map(|a| to_gossip(a)).collect()
}

pub fn to_auction_data(gossip: AuctionGossip) -> AuctionData {
    AuctionData {
        auction_id: gossip.get_auction_id().as_bytes().to_owned(),
        title: gossip.get_title(),
        seller: gossip.get_seller().as_bytes().to_owned(),
        price: gossip.get_price(),
        status: gossip.get_bool_state(),
    }
}

pub fn to_auction_data_vec(gossips: Vec<AuctionGossip>) -> Vec<AuctionData> {
    let convert = | gossip: &AuctionGossip| {
        AuctionData {
            auction_id: gossip.get_auction_id().as_bytes().to_owned(),
            title: gossip.get_title(),
            seller: gossip.get_seller().as_bytes().to_owned(),
            price: gossip.get_price(),
            status: gossip.get_bool_state(),
        }
    };

    gossips.iter().map(|g| convert(g)).collect()

}

pub fn encode_store(value: &AuctionData, key: NodeID) -> Vec<u8> {
    let mut databuf: Vec<u8> = Vec::new();
    value.encode(&mut databuf);
    let mut key_bytes = key.as_bytes().to_vec();
    
    databuf.append(&mut key_bytes);
    databuf
}

pub fn encode_fvalue(value: &HasValue, key: NodeID) -> Vec<u8> {
    let mut databuf: Vec<u8> = Vec::new();
    value.encode(&mut databuf);
    let mut key_bytes = key.as_bytes().to_vec();
    
    databuf.append(&mut key_bytes);
    databuf
}

pub fn gen_cookie() -> u64 {
    let mut rng = rand::thread_rng();
    let cookie: u64= rng.gen();
    cookie
}
