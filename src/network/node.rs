use std::{collections::VecDeque, borrow::Borrow};
use to_binary::BinaryString;

use crate::key::{Key};

const k_MAX_ENTRIES: usize = 20;
#[derive(Debug,Clone,PartialEq,Eq,Ord,PartialOrd)]
pub struct Contact {
    pub uid: Key,
    pub ip: String,
    pub port: u16,
}

impl Contact {
    pub fn new(uid: Key, ip: String, port: u16) -> Contact {
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
  pub left : Option<Box<Node>>,
  pub right : Option<Box<Node>>,
  pub bucket : Option<Bucket>,
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

    pub fn init_tree(&mut self, con: Contact,mut index: usize,mut chunk: usize) {
        // if we reach the leaf
        if chunk == 31 && index == 7 {
           let mut b = Bucket::new();
           b.insert(Box::new(con));
           self.bucket = Some(b);
           return;
        } else if index == 8 {
            chunk += 1;
            index = 0;
        } 
        let bits = BinaryString::from(con.uid.as_bytes()[chunk]);
        let mut n = Node::new();
        n.init_tree(con,index+1,chunk);

        match bits.0.chars().nth(index) {
            Some('0') => self.set_left(n),
            Some('1') => self.set_right(n),
            Some(_) => panic!("Invalid index"),
            None => panic!("Out of string bounds"),
        }
    }

    pub fn insert(&mut self,con: Contact, mut index: usize, mut chunk: usize) {
        if self.bucket.is_some() {
            if self.bucket.as_mut().unwrap().0.len() == 20 {
                //TODO function to ping a node to remove or keep in bucket
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
                 match self.left.as_mut() {
                     None =>{
                        let mut node = Node::new();
                        let mut b = Bucket::new();
                        b.insert(Box::new(con));
                        node.set_bucket(b);
                        self.set_left(node);
                     } ,
                     Some(node) => node.insert(con,index+1,chunk),
                 }
             },
             Some('1') => { 
                match self.right.as_mut() {
                    None => {
                        let mut node = Node::new();
                        let mut b = Bucket::new();
                        b.insert(Box::new(con));
                        node.set_bucket(b);
                        self.set_right(node);
                    },
                    Some(node) => node.insert(con,index+1,chunk),
                }
             },
             Some(_) => panic!("Invalid index"),
             None => panic!("Out of string bounds"),
         }
    }

    //returns a reference to the node containing the k-bucket for the id
    pub fn lookup(&self, id: Key, mut index: usize, mut chunk: usize) -> &Node {
        if chunk == 31 && index == 7 {
            return &self;
         } else if index == 8 {
             chunk += 1;
             index = 0;
         } 
         let bits = BinaryString::from(id.as_bytes()[chunk]);
         match bits.0.chars().nth(index) {
             Some('0') =>{
                 match &self.left {
                     None => {
                         println!("left");
                         self 
                        },
                     Some(node) => node.lookup(id,index+1,chunk),
                 }
             },
             Some('1') => { 
                match &self.right {
                    None => {
                        println!("right");
                        self
                    },
                    Some(node) => node.lookup(id,index+1,chunk),
                }
             },
             Some(_) => panic!("Invalid index"),
             None => panic!("Out of string bounds"),
         }
    }
    
}