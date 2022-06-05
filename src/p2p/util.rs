use primitive_types::H256;
use prost::Message;
use rand::Rng;
use tonic::Request;

use crate::{auctions::auction::{AuctionGossip, AuctionState}, ledger::block::{Block, Data}};

use super::{
    kademlia::{Kcontact, AuctionData, f_value_repl::HasValue, Gblock, Transaction, BroadcastReq, broadcast_req::Rdata},
    key:: NodeID,
    node::Contact,
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
        NodeID::from_vec(auction.buyer.clone()),
        auction.price,
        state(auction.status),
        NodeID::from_vec(auction.seller.clone())
    )
}

pub fn to_gossip_vec(auctions: Vec<AuctionData>) -> Vec<AuctionGossip> {
    auctions.iter().map(|a| to_gossip(a)).collect()
}

pub fn to_auction_data(gossip: AuctionGossip) -> AuctionData {
    AuctionData {
        auction_id: gossip.get_auction_id().as_bytes().to_owned(),
        title: gossip.get_title(),
        seller: gossip.get_seller().as_bytes().to_owned(),
        buyer: gossip.get_buyer().as_bytes().to_owned(),
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
            buyer: gossip.get_buyer().as_bytes().to_owned(),
            price: gossip.get_price(),
            status: gossip.get_bool_state(),
        }
    };

    gossips.iter().map(|g| convert(g)).collect()

}

pub fn encode_store(value: &AuctionData, key: NodeID) -> Vec<u8> {
    let mut databuf: Vec<u8> = Vec::new();
    let _ = value.encode(&mut databuf);
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

pub fn grpc_block(block: Block) -> Gblock {
    Gblock { 
        id: block.id, 
        nonce: block.nonce, 
        prev_hash: block.prev_hash.as_bytes().to_owned(), 
        current_hash: block.hash.as_bytes().to_owned(), 
        timestamp: block.timestamp, 
        data: Some(grpc_transaction(block.data)), 
    }
}

pub fn grpc_transaction(data : Data) -> Transaction {
    Transaction {
        seller: data.get_seller().as_bytes().to_owned(),
        buyer: data.get_buyer().as_bytes().to_owned(),
        amout: data.get_amount(),
        auction_id:data.get_auction_id().as_bytes().to_owned(),
    }
}

pub fn to_block(block: Gblock) -> Block {
    Block::new(
        block.id,
        block.nonce,
        H256::from_slice(block.prev_hash.as_slice()),
        H256::from_slice(block.current_hash.as_slice()),
        block.timestamp,
        to_data(block.data.unwrap())
    )
}

pub fn to_data(data: Transaction) -> Data {
    Data::new(
        NodeID::from_vec(data.buyer), 
        NodeID::from_vec(data.seller),
        data.amout,
        H256::from_slice(data.auction_id.as_slice())
    )
}

pub fn build_brequest(timestamp: &u64, data: &Rdata) -> Request<BroadcastReq> {
    Request::new(BroadcastReq {
        timestamp: timestamp.clone(),
        rdata: Some(data.clone()),
    })   
}