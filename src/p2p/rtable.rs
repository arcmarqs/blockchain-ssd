
use std::{collections::HashSet, cmp};
use super::{node::{Node,Bucket, Contact},
            kad::{KadNode},
            key::Key,
};
use to_binary::BinaryString;

#[derive(Debug,Default)]
pub struct Rtable{pub head: Node}

impl Rtable {
    pub fn new() -> Rtable {
        Rtable {
        head : Node::new(),
        }
    }

    //inserts the contact in the appropriated kbucket.
    pub fn insert(&mut self, con: Contact, uid: Key) {
        self.head.insert(con,uid,0,0);
    }

    //returns k closest nodes to the key
    pub fn lookup(&self, id: Key) -> Vec<Box<Contact>> {
        let mut index = 0;
        let mut chunk = 0;
        let mut k_closest = Vec::<Box<Contact>>::new();
        let mut target_id = id.clone();
        let mut visited_buckets = HashSet::new();
        while k_closest.len() <= 20 {
            let kbucket = self.head.lookup(target_id,index,chunk);
            if let Some(bucket) = kbucket {
                for contact in bucket.get_sorted(id).drain(0..) {
                   
                    k_closest.push(contact);
                    if k_closest.len() == 20 {
                        println!("final {:?}",k_closest);
                        return k_closest;
                    }
                }
            }
            target_id = target_id.set_bitmask(index,chunk);
            let prefix = target_id.prefix(index,chunk);  
            if visited_buckets.contains(&prefix) {
                break;
            }

            visited_buckets.insert(prefix);
            index = 0;
            chunk = 0;
        }
        println!("final {:?}",k_closest);
        k_closest
    } 
}