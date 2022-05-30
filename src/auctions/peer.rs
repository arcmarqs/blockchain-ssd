


pub struct AuctionPeer {
    node: KadNode,
    subscribed_auctions: HashMap<NodeId, Aution>,
    my_subscribers: HashMap<Aution, Vec<NodeId>>,
    known_brokers: HashMap<NodeId,>
}