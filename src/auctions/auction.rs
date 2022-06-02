use chrono::{DateTime, Duration, Utc};
use openssl::sha::Sha256;
use primitive_types::H256;

use crate::p2p::key::NodeID;

#[derive(Debug,Copy,Clone)]
enum AuctionState {
    ONGOING,
    FINISHED,
}

#[derive(Debug,Clone)]
pub struct Auction {
    title: String,
    seller: NodeID,
    initial_price: f32,
    current_price: f32,
    highest_bidder: Option<NodeID>,
    starting_time: DateTime<Utc>,
    time_remaining: Duration,
    state: AuctionState,
    auction_id: H256,
}

impl Auction{
    pub fn new(title: String, seller: NodeID, initial_price: f32, time: i64)-> Auction {
        let starting_time = DateTime::from(Utc::now());
        let auction_id = gen_auction_id(&title,seller,starting_time);
        Auction{
            title,
            seller,
            initial_price,
            current_price : initial_price,
            highest_bidder : None,
            starting_time,
            time_remaining: Duration::hours(time),
            state: AuctionState::ONGOING,
            auction_id,
        }
    }

    pub fn get_title(&self) -> &String {
        &self.title
    }

    pub fn get_seller(&self) -> NodeID {
        self.seller
    }

    pub fn get_initial_price(&self) -> f32 {
        self.initial_price
    }
    pub fn get_current_price(&self) -> f32 {
        self.current_price
    }

    pub fn get_highest_price(&self) -> Option<NodeID> {
        self.highest_bidder
    }

    pub fn get_starting_time(&self) -> DateTime<Utc> {
        self.starting_time
    }

    pub fn get_time_remaining(&self) -> Duration {
        self.time_remaining
    }
    
    pub fn get_auction_id(&self) -> H256 {
        self.auction_id
    }

    pub fn set_current_price(&mut self, x: f32) { 
        self.current_price = x;
    }

    pub fn bid(&mut self, bid_amout: f32, bidder: NodeID) -> Result<(),&str> {
        if self.current_price >= bid_amout {
            Err("bid must be greater than current price")
        } else {
            self.current_price = bid_amout;
            self.highest_bidder = Some(bidder);
            Ok(())
        }
    } 
}

fn gen_auction_id(title: &String, seller: NodeID, start: DateTime<Utc>) -> H256 {
    let mut hasher = Sha256::new();
    hasher.update(title.as_bytes());
    hasher.update(seller.as_bytes());
    hasher.update(start.to_string().as_bytes());
    H256::from(hasher.finish())

}
