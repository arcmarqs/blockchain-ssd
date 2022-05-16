use std::{collections::VecDeque, borrow::Borrow};
use crate::key::{Key};

const k_MAX_ENTRIES: usize = 20;
#[derive(Debug,Clone,PartialEq,Eq,Ord,PartialOrd)]
pub struct Contact {
    uid: Key,
    ip: String,
    port: u8,
}

impl Contact {
    pub fn new(uid: Key, ip: String, port: u8) -> Contact {
        Contact {
            uid,
            ip,
            port,
        }
    }
}
#[derive(Debug,Clone)]
pub struct Bucket(VecDeque<Box<Contact>>);

impl Bucket {
    pub fn new() -> Bucket {
        Bucket(VecDeque::with_capacity(k_MAX_ENTRIES))
    }

    pub fn insert(&mut self, node: Box<Contact>) {
        self.0.push_back(node);
    }

    pub fn remove(&mut self, node: Box<Contact>) {
        let mut count = 0;
        for i in self.0.iter() {
            if &node ==  i{
                self.0.remove(count);
                self.0.shrink_to_fit();
                break;
            }
            count +=1;
        }
    }

    pub fn move_to_tail(&mut self, node: Box<Contact>) {
        let mut count = 0;
        let mut n : Option<Box<Contact>> = None;
        for i in self.0.iter() {
            if &node ==  i{
                n = self.0.remove(count);
                self.0.shrink_to_fit();
                break;
            }
            count +=1;
        }

        match n {
            None => panic!("Node {:?} does not exist in this bucket", node),
            Some(rnode) => self.insert(rnode),
        }
    }
}
#[derive(Debug,Clone)]
pub struct Node {
   left : Option<Box<Node>>,
   right : Option<Box<Node>>,
   bucket : Option<Bucket>,
} 

impl Node {
    pub fn new() -> Node {
        Node { 
            left: None, 
            right: None, 
            bucket: None
        }
    }
    
    #[inline]
    fn set_left(&mut self, left: Node) {
        self.left = Some(Box::new(left));
    }

    #[inline]
    fn set_right(&mut self, right: Node) {
        self.right = Some(Box::new(right));
    }

    #[inline]
    fn set_bucket(&mut self, bucket: Bucket) {
        self.bucket = Some(bucket);
    }

    #[inline]
    fn get_left(&self) -> Option<&Box<Node>> {
        self.left.as_ref()
    }

    #[inline]
    fn get_right(&self) -> Option<&Box<Node>> {
        self.right.as_ref()
    }

    #[inline]
    fn get_bucket(&self) -> Option<&Bucket> {
        self.bucket.as_ref()
    }

    #[inline]
    fn get_mut_bucket(&mut self) -> Option<&mut Bucket> {
        self.bucket.as_mut()
    }
}