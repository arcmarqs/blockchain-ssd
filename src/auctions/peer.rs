use primitive_types::H256;

use crate::ledger::block::Data;
use crate::p2p::client::Client;
use crate::p2p::kad::KadNode;
use crate::p2p::key::NodeID;
use std::collections::HashMap;
use std::sync::Arc;
use super::auction::{Auction,  AuctionGossip, Slotmap};

#[derive(Debug,Clone)]
pub struct AuctionPeer {
    pub client: Client,
    subscribed_auctions: HashMap<NodeID, Vec<AuctionGossip>>,
    my_auctions: HashMap<H256, Vec<NodeID>>,
    known_auctions: Slotmap,
}

impl AuctionPeer{
    pub fn new(node : Arc<KadNode>) -> AuctionPeer {
            AuctionPeer{
                client : Client::new(node),
                subscribed_auctions: HashMap::new(),
                my_auctions: HashMap::new(),
                known_auctions: Slotmap::new(),
        }
    }

    // only an exemple to test the blockchain
    pub fn fulfill_transaction(&self,index: i32) {
        let id = self.known_auctions.get(index).unwrap().clone();
        if self.my_auctions.contains_key(&id.get_auction_id()){
        let data = Data::from_auction(id);
        let _ = self.client.broadcast_transaction(data);
        }
    }

    pub fn new_auction(&mut self, title: String, duration : i64, initial_value: f32) {
        let auction = Auction::new(title,self.client.get_uid(), duration, initial_value);
        let auction_subscribers: Vec<NodeID> = Vec::new();
        let _ = self.client.annouce_auction(auction.to_gossip());
        self.my_auctions.insert(auction.get_auction_id(),auction_subscribers);
    }

    pub async fn get_avaliable_auctions(&mut self) {
        match self.client.get_avaliable_auctions().await {
            Some(vec) => {
                for gossip in vec {
                    self.known_auctions.insert(gossip);
                }

                println!("{:?}", self.known_auctions);
            },
            None => println!("No auctions avaliable"),
        }
    }

    pub async  fn bid_auction(&mut self, index: i32,bid : f32)  {
        match self.known_auctions.get_mut(index) {
            Some(gossip) =>{ 
                if let Ok(bidded_gossip) = gossip.bid(bid,self.client.get_uid()) {
                let _ = self.insert_subscribe(bidded_gossip.get_seller(), bidded_gossip.clone()).await;
                }
            },
            None => println!("Invalid auction"),
        }
    }
    
    pub async fn update_subscribed(&self) {
        let mut interesting_auctions: Vec<AuctionGossip>= Vec::new();
        for key in self.subscribed_auctions.keys() {
           if let Some(aucts) = self.client.send_fvalue(*key).await {
                for auct in aucts {
                    if self.subscribed_auctions.get(key).unwrap().contains(&auct) {
                            interesting_auctions.push(auct);
                    }
                }
            }
        }

        for auction in interesting_auctions {
            println!("{:?}",auction);
        }
    }
    

    async fn insert_subscribe(&mut self, key: NodeID, value: AuctionGossip) {
        match self.subscribed_auctions.get_mut(&key) {
            Some(vec) => {
                let mut id = 0;
                for v in vec.iter() {
                    if v == &value {
                    vec.remove(id);
                    break;
                    }
                    id +=1;
                }
                vec.push(value.clone());

            },
            None => {
                self.subscribed_auctions.insert(key, vec![value.clone()]);
            },
        }

        let _ = self.client.subscribe_auction(value.clone()).await;
    }
}
