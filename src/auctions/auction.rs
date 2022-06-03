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
    auction_id: H256,
    state: AuctionState,
    info: AuctionInfo,
}

impl Auction {
    pub fn new( title: String, seller: NodeID,duration : i64, initial_value: f32)  -> Auction {
        let starting_time = DateTime::from(Utc::now());
        let info = AuctionInfo::new(title,seller,starting_time,initial_value,duration);
        let auction_id = gen_auction_id(&title,seller,starting_time);
        Auction {
            auction_id,
            state: AuctionState::ONGOING,
            info
        }
    }

    pub fn bid(&mut self, bid_amout: f32, bidder: NodeID) -> Result<(),&str> {
        self.info.bid(bid_amout, bidder)
    } 
}
#[derive(Debug,Clone)]
pub struct AuctionInfo {
    title: String,
    seller: NodeID,
    initial_price: f32,
    current_price: f32,
    highest_bidder: Option<NodeID>,
    starting_time: DateTime<Utc>,
    time_remaining: Duration,
}

impl AuctionInfo {
    pub fn new(title: String, seller: NodeID,starting_time: DateTime<Utc>, initial_price: f32, time: i64)-> AuctionInfo {
        AuctionInfo{
            title,
            seller,
            initial_price,
            current_price : initial_price,
            highest_bidder : None,
            starting_time,
            time_remaining: Duration::hours(time),
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

    pub fn get_highest_bidder(&self) -> Option<NodeID> {
        self.highest_bidder
    }

    pub fn get_starting_time(&self) -> DateTime<Utc> {
        self.starting_time
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
