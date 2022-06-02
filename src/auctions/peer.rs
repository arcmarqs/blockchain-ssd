use crate::p2p::kad::KadNode;
use crate::p2p::key::NodeID;
use std::collections::HashMap;

use super::auction::Auction;

pub struct AuctionPeer {
    node: KadNode,
    subscribed_auctions: HashMap<NodeID, Auction>,
    my_subscribers: HashMap<Auction, Vec<NodeID>>,
}
