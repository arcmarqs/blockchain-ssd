use crate::p2p::client::Client;
use crate::p2p::kad::KadNode;
use crate::p2p::key::NodeID;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use super::auction::{Auction, AuctionInfo};

#[derive(Debug,Clone)]
pub struct AuctionPeer {
    client: Client,
    subscribed_auctions: HashMap<NodeID, Auction>,
    my_auctions: HashMap<Auction, Vec<NodeID>>,
}

impl AuctionPeer{
    pub fn new(node : Arc<KadNode>) -> AuctionPeer {
            AuctionPeer{
                client : Client::new(node),
                subscribed_auctions: HashMap::new(),
                my_auctions: HashMap::new(),
        }
    }

    pub fn new_auction(&mut self, title: String, duration : i64, initial_value: f32) {
        let auction = Auction::new(title,self.client.get_uid(), duration, initial_value);
        self.my_auctions.insert(auction,Vec::new());
    }

    pub fn get_subscribed_auctions(&self) -> &HashMap<NodeID, Auction> { 
        &self.subscribed_auctions 
    }

    pub fn get_my_auctions(&self) -> &HashMap<Auction, Vec<NodeID>> { 
        &self.my_auctions 
    }

    pub fn set_subscribed_auctions(&mut self, x:NodeID, a:Auction) { 
        self.subscribed_auctions.insert(x,a);
    }

    /*fn set_my_subscribers(&mut self, a:Auction, x:Vec<NodeID>>) { 
        //verificar o auction e depois a inserção nodeId
    }*/
}

impl Hash for AuctionPeer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.subscribed_auctions.hash(state);
        self.my_auctions.hash(state);
    }

}
impl PartialEq for AuctionPeer {
    fn eq(&self, other: &Self) -> bool {
        self.client.get_uid() == other.client.get_uid()
    }
}

impl Eq for AuctionPeer {}