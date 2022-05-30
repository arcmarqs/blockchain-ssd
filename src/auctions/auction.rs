use chrono::{DateTime, Utc, Duration};

use crate::p2p::key::NodeID;

enum AuctionState {
    ONGOING,
    FINISHED,
}

pub struct Auction {
    title: String,
    seller: NodeID,
    initial_price: f32,
    current_price: f32,
    highest_price: NodeID,
    starting_time: DateTime<Utc>,
    time_remaining: Duration,
    state: AuctionState,
    auction_id: String,
}