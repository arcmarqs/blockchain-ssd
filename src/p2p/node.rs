use std::collections::VecDeque;

use chrono::{DateTime, Utc};
use futures::executor;
use to_binary::BinaryString;

use super::{key::{NodeID, NodeValidator}, K_MAX_ENTRIES, kademlia::Kcontact, client::send_ping};



#[derive(Debug,Clone,Hash,PartialEq,Eq,Ord,PartialOrd)]
enum LastSeen {
    Never,
    Seen(DateTime<Utc>),
}

#[derive(Debug,Clone,Hash,PartialEq,Eq,Ord,PartialOrd)]
pub struct Contact {
    pub(crate) uid: NodeID,
    pub(crate) address: String,
    last_seen: LastSeen,
    pub_key: Vec<u8>,
}

impl Contact {
    pub fn new(uid: NodeID, address: String, pub_key: Vec<u8>) -> Contact {
        Contact {
            uid,
            address,
            pub_key,
            last_seen: LastSeen::Never
        }
    }

    pub fn get_pubkey(&self) -> &[u8] {
        self.pub_key.as_ref()
    }

    pub fn as_kcontact(&self) -> Kcontact {
        Kcontact {
            uid: self.uid.as_bytes().to_owned(),
            address: self.address.clone(),
            pub_key: self.pub_key.to_owned(),
        }
    }

    pub fn see(&mut self) {
        self.last_seen = LastSeen::Seen(Utc::now());
    }
}

#[derive(Debug,Default,Clone)]
pub struct Bucket(VecDeque<Box<Contact>>);

impl Bucket {
    pub fn new() -> Bucket {
        Bucket(VecDeque::with_capacity(K_MAX_ENTRIES))
    }

    pub fn insert(&mut self,mut node: Box<Contact>) {
        let mut index = 0;
        for i in self.0.iter() {
            if &node ==  i{
                break;
            }
            index +=1;
        }

        match self.0.get_mut(index) {
            Some(index) => {
                index.see();
                self.move_to_tail(node)
            },
            None => {
                node.as_mut().see();
                self.0.push_back(node);
            },
        }
    }

    pub fn remove(&mut self, node: Box<Contact>) {
        let mut count = 0;
        for i in self.0.iter() {
            if &node ==  i{
                self.0.remove(count);
                break;
            }
            count +=1;
        }
    }

    pub fn move_to_tail(&mut self, node: Box<Contact>) {
        let mut count = 0;
        for i in self.0.iter() {
            if &node ==  i{
                self.0.swap(count,self.0.len()-1);
                break;
            }
            count +=1;
        }
    }

    pub fn split(&mut self,id: NodeID,index: usize,chunk: usize) -> (Bucket,Bucket) {
        let mut bucket0 = Bucket::new();
        let mut bucket1 = Bucket::new();
        let byte = id.as_bytes()[chunk];
        for con in self.0.drain(0..) {
            let con_byte = con.uid.as_bytes()[chunk];
            let bits = BinaryString::from(con_byte ^ byte);
            match bits.0.chars().nth(index) {
                Some('1') => bucket1.insert(con),
                Some('0') => bucket0.insert(con),
                _ => panic!("Invalid bit"),
            };
        }

        (bucket1,bucket0)
    }

    pub fn is_full(&self) -> bool {
        self.0.len() == K_MAX_ENTRIES
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    // returns the vector sorted by increasing distance to the given key
    pub fn get_sorted(&self, id: NodeID) -> Vec<Box<Contact>> {
        let dist = |a:&Box<Contact>, b: &Box<Contact>| {
            id.distance(a.uid).partial_cmp(&id.distance(b.uid))
        };

        let mut vec = Vec::from(self.0.clone());
        vec.sort_by(|a,b| dist(a,b).unwrap());
        vec
    }

    pub fn insert_full(&mut self,my_address: &str,validator: &NodeValidator, mut con : Box<Contact>) {
        let pinged = executor::block_on(send_ping(my_address,validator, *self.0.front().unwrap().clone()));
        match pinged {
            true => (),
            false => {
                self.0.pop_front();
                con.see();
                self.0.push_back(con);
            },
        }
    }
}

#[derive(Debug,Default,Clone)]
pub struct Node {
  pub left : Option<Box<Node>>,
  pub right : Option<Box<Node>>,
  pub bucket : Option<Bucket>,
} 

impl Node {
    pub fn new() -> Node {
        Node { 
            left: None, 
            right: None, 
            bucket: Some(Bucket::new()),
        }
    }
    
    #[inline]
    pub fn set_left(&mut self, left: Node) {
        self.left = Some(Box::new(left));
    }

    #[inline]
    pub fn set_right(&mut self, right: Node) {
        self.right = Some(Box::new(right));
    }

    #[inline]
    fn set_bucket(&mut self, bucket: Bucket) {
        self.bucket = Some(bucket);
    }

    #[inline]
    pub fn get_left(&self) -> Option<&Box<Node>> {
        self.left.as_ref()
    }

    #[inline]
    pub fn get_right(&self) -> Option<&Box<Node>> {
        self.right.as_ref()
    }

    #[inline]
    pub fn get_bucket(&self) -> Option<&Bucket> {
        self.bucket.as_ref()
    }

    #[inline]
    pub fn get_mut_bucket(&mut self) -> Option<&mut Bucket> {
        self.bucket.as_mut()
    }

    pub fn insert(&mut self,my_address: &str,con: Contact,validator: &NodeValidator, mut index: usize, mut chunk: usize) {
        if self.bucket.is_some() {
            if self.bucket.as_ref().unwrap().is_full() {
                //checking the range of the node  [87,234,234,]
                let con_byte = con.uid.as_bytes()[chunk];
                let byte = validator.get_nodeid().as_bytes()[chunk];
                let bits = BinaryString::from(con_byte ^ byte);
                match bits.0.chars().nth(index) {
                    Some('1') => {
                        //don't split buckets into buckets
                        self.bucket.as_mut().unwrap().insert_full(my_address,validator,Box::new(con));
                        return;
                    },
                    Some('0') => {
                        let (b1,mut b0) = self.bucket.as_mut().unwrap().split(validator.get_nodeid(),index,chunk);
                        b0.insert(Box::new(con));
                        let mut outrange = Node::new();
                        let mut inrange = Node::new();
                        outrange.set_bucket(b1);
                        inrange.set_bucket(b0);
                        let matched_bit = BinaryString::from(byte).0.chars().nth(index);
                        match matched_bit {
                            Some('1') =>{ 
                                self.set_right(inrange);
                                self.set_left(outrange);
                            }
                            Some('0') =>{
                                self.set_right(outrange);
                                self.set_left(inrange);
                             }
                            _ => panic!("Invalid bit"),
                        }
                        self.bucket = None;
                    }
                    _ => panic!("Invalid bit"),
                }
                return;
            }
            self.bucket.as_mut().unwrap().insert(Box::new(con));
            return;
        } else if chunk == 31 && index == 7 {
            let mut b = Bucket::new();
            b.insert(Box::new(con));
            self.bucket = Some(b);
            return;
         } else if index == 8 {
             chunk += 1;
             index = 0;
         } 
         
         let bits = BinaryString::from(con.uid.as_bytes()[chunk]);
         match bits.0.chars().nth(index) {
             Some('0') =>{
                 if let Some(node) = self.left.as_mut() {
                    node.insert(my_address,con,validator,index+1,chunk)
                } else {
                    let mut node = Node::new();
                    let mut b = Bucket::new();
                    b.insert(Box::new(con));
                    node.set_bucket(b);
                    self.set_left(node);
                }
             },
             Some('1') => { 
                if let Some(node) = self.right.as_mut() {
                    node.insert(my_address,con,validator,index+1,chunk)
                } else {
                    let mut node = Node::new();
                    let mut b = Bucket::new();
                    b.insert(Box::new(con));
                    node.set_bucket(b);
                    self.set_right(node);
                }
             },
             Some(_) => panic!("Invalid index"),
             None => panic!("Out of string bounds"),
         }
    }

    //returns a reference to the node containing the k-bucket for the id
    pub fn lookup(&self, id: NodeID, mut index: usize, mut chunk: usize) -> Option<&Bucket> {
        if chunk == 31 && index == 7 {
            return self.bucket.as_ref();
         } else if index == 8 {
             chunk += 1;
             index = 0;
         } 
         let bits = BinaryString::from(id.as_bytes()[chunk]);
         match bits.0.chars().nth(index) {
             Some('0') =>{
                 match &self.left {
                     None => {
                         self.bucket.as_ref() 
                        },
                     Some(node) => {
                         node.lookup(id,index+1,chunk)
                        },
                 }
             },
             Some('1') => { 
                match &self.right {
                    None => {
                        self.bucket.as_ref()
                    },
                    Some(node) =>{
                        node.lookup(id,index+1,chunk)
                    },
                }
             },
             Some(_) => panic!("Invalid index"),
             None => panic!("Out of string bounds"),
         } 
    }  
}
