use crate::p2p::kad::KadNode;
use crate::p2p::key::NodeID;
use std::collections::HashMap;

use super::auction::Auction;

pub struct AuctionPeer {
    node: KadNode,
    subscribed_auctions: HashMap<NodeID, Auction>,
    my_subscribers: HashMap<Auction, Vec<NodeID>>,
}

impl AuctionPeer{
    pub fn new(node: KadNode) -> AuctionPeer {
            AuctionPeer{
                node,
                subscribed_auctions: HashMap::new(),
                my_subscribers: HashMap::new(),
        }
    }
    fn get_node(&self) -> &KadNode { &self.node }
    fn get_subscribed_auctions(&self) -> &HashMap<NodeID, Auction> { &self.subscribed_auctions }
    fn get_my_subscribers(&self) -> &HashMap<Auction, Vec<NodeID>> { &self.my_subscribers }

    fn set_subscribed_auctions(&mut self, x:NodeID, a:Auction) { 
        self.subscribed_auctions.insert(x,a);
    }

    /*fn set_my_subscribers(&mut self, a:Auction, x:Vec<NodeID>>) { 
        //verificar o auction e depois a inserção nodeId
    }*/
}
