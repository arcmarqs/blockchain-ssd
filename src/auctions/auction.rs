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
    pub fn new(title: String, seller: NodeID, initial_price: f32, current_price: f32, highest_bidder: NodeID, time_remaining: Duration,state: AuctionState,
        auction_id: String)-> Auction {
        Auction{
            title,
            seller,
            initial_price,
            current_price,
            highest_bidder,
            starting_time: DateTime::new(Utc::now()),
            time_remaining,
            state,
            auction_id
        }
    }

    pub fn get_title(&self) -> String {self.title}
    pub fn get_seller(&self) -> NodeID {self.seller}
    pub fn get_initial_price(&self) -> f32 {self.initial_price}
    pub fn get_current_price(&self) -> f32 {self.current_price}
    pub fn get_highest_price(&self) -> NodeID {self.highest_price}
    pub fn get_starting_time(&self) -> DateTime<Utc> {self.starting_time}
    pub fn get_time_remaining(&self) -> Duration {self.time_remaining}
    pub fn get_state(&self) -> AuctionState {self.state}
    pub fn get_auction_id(&self) -> String {self.auction_id}

    pub fn set_title(&mut self, x: String) { self.title = x;}

    pub fn set_seller(&mut self, x: NodeID) { self.seller = x;}

    pub fn set_initial_price(&mut self, x: f32) { self.initial_price = x;}

    pub fn set_current_price(&mut self, x: f32) { self.current_price = x;}
    
    pub fn set_highest_price(&mut self, x: NodeID) { self.highest_price = x;}

    pub fn set_starting_time(&mut self, x: Utc) {
         self.starting_time.insert(x);
    }
    pub fn set_time_remaining(&mut self, x: Duration) { self.time_remaining = x;}
    pub fn set_state(&mut self, x: AuctionState) { self.state = x;}
    pub fn set_auction_id(&mut self, x: String) { self.auction_id = x;}
}
