use chrono::{DateTime, Duration, Utc};
use openssl::sha::Sha256;
use primitive_types::H256;
use std::{hash::Hash, collections::BTreeMap};

use crate::p2p::key::NodeID;

#[derive(Debug,Copy,Clone)]
pub enum AuctionState {
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
        let auction_id = gen_auction_id(&title,seller,starting_time);
        let info = AuctionInfo::new(title,seller,starting_time,initial_value,duration);
        Auction {
            auction_id,
            state: AuctionState::ONGOING,
            info
        }
    }

    pub fn get_auction_id(&self) -> H256 {
        self.auction_id.clone()
    }
/*  UNUSED
    pub fn bid(&mut self, bid_amout: f32, bidder: NodeID) -> Result<(),&str> {
        self.info.bid(bid_amout, bidder)
    } 
*/
    pub fn to_gossip(&self) -> AuctionGossip {
        AuctionGossip::from_auction(self)
    }
}

impl Hash for Auction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.auction_id.hash(state)
    }
}

impl PartialEq for Auction {
    fn eq(&self, other: &Self) -> bool {
        self.auction_id == other.auction_id
    }
}

impl Eq for Auction {}

#[derive(Debug,Clone)]
pub struct AuctionInfo {
    title: String,
    seller: NodeID,
    _initial_price: f32,
    current_price: f32,
    highest_bidder: Option<NodeID>,
    _starting_time: DateTime<Utc>,
    _time_remaining: Duration,
}

impl AuctionInfo {
    pub fn new(title: String, seller: NodeID, _starting_time: DateTime<Utc>, _initial_price: f32, time: i64)-> AuctionInfo {
        AuctionInfo{
            title,
            seller,
            _initial_price,
            current_price : _initial_price,
            highest_bidder : None,
            _starting_time,
            _time_remaining: Duration::hours(time),
        }
    }

    pub fn get_title(&self) -> &String {
        &self.title
    }
/* UNUSED 
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

    pub fn bid(&mut self, bid_amout: f32, bidder: NodeID) -> Result<(),&'static str> {
        if self.current_price >= bid_amout {
            Err("bid must be greater than current price")
        } else {
            self.current_price = bid_amout;
            self.highest_bidder = Some(bidder);
            Ok(())
        }
    } 

*/
}

#[derive(Debug,Clone)]
pub struct AuctionGossip {
    auction_id: H256,
    title: String,
    seller: NodeID,
    buyer: NodeID,
    current_price: f32,
    state: AuctionState,
}

impl AuctionGossip{
    pub fn from_auction(auction: &Auction) -> AuctionGossip {
        AuctionGossip {
            auction_id: auction.auction_id,
            title: auction.info.get_title().to_owned(),
            seller: auction.info.seller,
            buyer: auction.info.seller,
            current_price: auction.info.current_price,
            state: auction.state,
        }
    }

    pub fn new(auction_id: H256, title: String, buyer: NodeID,current_price: f32, state: AuctionState, seller: NodeID) -> AuctionGossip {
        AuctionGossip {
            auction_id,
            state,
            title,
            buyer,
            seller,
            current_price,
        }
    }

    pub fn get_title(&self) -> String {
        self.title.clone()
    }

    pub fn get_seller(&self) -> NodeID {
        self.seller.clone()
    }

    pub fn get_price(&self) -> f32 {
        self.current_price
    }

    pub fn get_auction_id(&self) -> H256 {
        self.auction_id
    }

    pub fn get_buyer(&self) -> NodeID {
        self.buyer
    }

    pub fn get_bool_state(&self) -> bool {
        match self.state {
            AuctionState::ONGOING => true,
            AuctionState::FINISHED => false,
        }
    }

    pub fn bid(&mut self, bid_amout: f32,buyer: NodeID) -> Result<AuctionGossip,&'static str> {
        if self.current_price >= bid_amout {
            Err("bid must be greater than current price")
        } else {
            self.current_price = bid_amout;
            self.buyer = buyer.clone();
            Ok(self.clone())
        }
    }
}

impl Hash for AuctionGossip {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.auction_id.hash(state);
    }
}

impl PartialEq for AuctionGossip {
    fn eq(&self, other: &Self) -> bool {
        self.auction_id == other.auction_id 
    }
}

impl Eq for AuctionGossip {}

fn gen_auction_id(title: &String, seller: NodeID, start: DateTime<Utc>) -> H256 {
    let mut hasher = Sha256::new();
    hasher.update(title.as_bytes());
    hasher.update(seller.as_bytes());
    hasher.update(start.to_string().as_bytes());
    H256::from(hasher.finish())

}

#[derive(Debug,Clone)]
pub struct Slotmap {
    map: BTreeMap<i32,AuctionGossip>,
    index: i32,
}

impl Slotmap {
    pub fn new() -> Slotmap {
        Slotmap {
            map: BTreeMap::new(),
            index: -1,
        }
    }

    pub fn get_mut(&mut self,index: i32) -> Option<&mut AuctionGossip> {
        self.map.get_mut(&index)
    }

    pub fn get(&self,index: i32) -> Option<&AuctionGossip> {
        self.map.get(&index)
    }

    pub fn insert(&mut self, gossip: AuctionGossip) -> i32 {
        let mut count = 0;
        for val in self.map.values_mut() {
            if &gossip == val {
                self.map.remove(&count);
                self.map.insert(count, gossip);
                return count;
            }
            count +=1;
        }
        self.index += 1;
        self.map.insert(self.index, gossip);
        self.index
    }
/*  UNUSED
    pub fn remove(&mut self, gossip: AuctionGossip) -> Option<AuctionGossip> {
        let mut count = 0;

        for val in self.map.values() {
          if val == &gossip {
              break;
          }
            count +=1;
        }
        self.map.remove(&count)
    }

    pub fn lookup_value(&self, gossip: AuctionGossip) -> Option<i32>{
        let mut count = 0;

        for val in self.map.values() {
          if val == &gossip {
              return Some(count);
          }
            count +=1;
        }
        None
    }
    */
}