use crate::p2p::kad::KadNode;
use std::collections::{HashMap};
use crate::p2p::key::NodeID;

use super::auction::Auction;

pub struct AuctionPeer {
    node: KadNode,
    subscribed_auctions: HashMap<NodeID, Auction>,
    my_subscribers: HashMap<Auction, Vec<NodeID>>,
}
impl AuctionPeer{
    pub fn new(node: KadNode, id_sa: NodeID, auction_s: Auction, id_ms: NodeID, auction_m: Auction) -> AuctionPeer {
            AuctionPeer{
                node,
                subscribed_auctions: HashMap::new(id_sa, auction_s)
                my_subscribers: HashMap::new(auction_m, Vec::new(id_ms)),
        }
    }
    fn get_node(&mut self) -> KadNode {return node;}
    fn get_subscribed_auctions(&mut self) -> HashMap<NodeID, Auction> {return subscribed_auctions;}
    fn get_my_subscribers(&mut self) -> HashMap<Auction, Vec<NodeID>> {return my_subscribers;}
    
    fn set_node(&mut self, x: KadNode) {
        self.node = x;
    }
    fn set_subscribed_auctions(&mut self, x:NodeID, a:Auction>) { 
        self.subscribed_auctions.insert(x,a);
    }

    /*fn set_my_subscribers(&mut self, a:Auction, x:Vec<NodeID>>) { 
        //verificar o auction e depois a inserção nodeId
    }*/
}
