use crate::p2p::kad::KadNode;
use std::collections::{HashMap};
use crate::p2p::key::NodeID;

use super::auction::Auction;

pub struct AuctionPeer {
    node: KadNode,
    subscribed_auctions: HashMap<NodeID, Auction>,
    my_subscribers: HashMap<Auction, Vec<NodeID>>,
}
