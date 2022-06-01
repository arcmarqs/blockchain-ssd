use std::collections::HashSet;

use super::{key::{NodeValidator, NodeID}, node::{Node, Contact}, K_MAX_ENTRIES};


#[derive(Debug,Default)]
pub struct Rtable{pub head: Node}

impl Rtable {
    pub fn new() -> Rtable {
        Rtable {
        head : Node::new(),
        }
    }

    //inserts the contact in the appropriated kbucket.
    pub fn insert(&mut self, con: Contact, validator: &NodeValidator) {
        self.head.insert(con,validator,0,0);
    }

    //returns k closest nodes to the key
    pub fn lookup(&self, id: NodeID) -> Vec<Box<Contact>> {
        let mut index = 0;
        let mut chunk = 0;
        let mut k_closest = Vec::<Box<Contact>>::new();
        let mut target_id = id.clone();
        let mut visited_buckets = HashSet::new();
        while k_closest.len() <= K_MAX_ENTRIES {
            let kbucket = self.head.lookup(target_id,index,chunk);
            if let Some(bucket) = kbucket {
                let mut bucket = bucket.get_sorted(id);       
                for contact in bucket.drain(0..) {
                    if k_closest.contains(&contact){
                        break;
                    }
                    k_closest.push(contact);
                    if k_closest.len() == K_MAX_ENTRIES {
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
        k_closest
    } 
}
