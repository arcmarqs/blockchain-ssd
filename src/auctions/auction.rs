use chrono::{DateTime, Duration, Utc};

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
    highest_bidder: NodeID,
    starting_time: DateTime<Utc>,
    time_remaining: Duration,
    state: AuctionState,
    auction_id: String,
}

impl Auction{
    pub fn new(title: String, seller: NodeID, initial_price: f32, current_price: f32, highest_bidder: NodeID, time_remaining: Duration,
        auction_id: String)-> Auction {
            let date = DateTime::from(Utc::now());
        Auction{
            title,
            seller,
            initial_price,
            current_price,
            highest_bidder,
            starting_time: date,
            time_remaining,
            state: AuctionState::ONGOING,
            auction_id,
        }
    }

    pub fn get_seller(&self) -> NodeID {self.seller}
    pub fn get_initial_price(&self) -> f32 {self.initial_price}
    pub fn get_current_price(&self) -> f32 {self.current_price}
   
    pub fn set_time_remaining(&mut self, x: Duration) { self.time_remaining = x;}
    pub fn set_auction_id(&mut self, x: String) { self.auction_id = x;}
}
