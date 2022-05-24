use std::collections::VecDeque;
use to_binary::BinaryString;
use super::{key::{Key}, protocol::kademlia::Kcontact};

const K_MAX_ENTRIES: usize = 20;

#[derive(Debug,Default,Clone,PartialEq,Eq,Ord,PartialOrd)]
pub struct Contact {
    pub(crate) uid: Key,
    pub(crate) ip: String,
    pub(crate) port: u16,
}

impl Contact {
    pub fn new(uid: Key, ip: String, port: u16) -> Contact {
        Contact {
           uid: uid,
            ip,
            port: port,
        }
    }

    pub fn as_kcontact(&self) -> Kcontact {
        Kcontact {
            uid: self.uid.as_bytes().to_owned(),
            ip: self.ip.clone(),
            port: self.port as i32,
        }
    }
}

#[derive(Debug,Default,Clone)]
pub struct Bucket(VecDeque<Box<Contact>>);

impl Bucket {
    pub fn new() -> Bucket {
        Bucket(VecDeque::with_capacity(K_MAX_ENTRIES))
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

    pub fn split(&mut self,id: Key,index: usize,chunk: usize) -> (Bucket,Bucket) {
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
        self.0.len() == 20
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    // returns the vector sorted by increasing distance to the given key
    pub fn get_sorted(&self, id: Key) -> Vec<Box<Contact>> {
        let dist = |a:&Box<Contact>, b: &Box<Contact>| {
            id.distance(a.uid).partial_cmp(&id.distance(b.uid))
        };

        let mut vec = Vec::from(self.0.clone());
        vec.sort_by(|a,b| dist(a,b).unwrap());
        vec
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

    pub fn insert(&mut self,con: Contact,id: Key, mut index: usize, mut chunk: usize) {
        if self.bucket.is_some() {
            if self.bucket.as_ref().unwrap().is_full() {
                //checking the range of the node  [87,234,234,]
                let con_byte = con.uid.as_bytes()[chunk];
                let byte = id.as_bytes()[chunk];
                let bits = BinaryString::from(con_byte ^ byte);
                match bits.0.chars().nth(index) {
                    Some('1') => {
                        //don't split buckets into buckets
                        println!("full bucket not split");
                    }
                    Some('0') => {
                        let (b1,mut b0) = self.bucket.as_mut().unwrap().split(id,index,chunk);
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
                 match self.left.as_mut() {
                     None =>{
                        let mut node = Node::new();
                        let mut b = Bucket::new();
                        b.insert(Box::new(con));
                        node.set_bucket(b);
                        self.set_left(node);
                     } ,
                     Some(node) => node.insert(con,id,index+1,chunk),
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
                    Some(node) => node.insert(con,id,index+1,chunk),
                }
             },
             Some(_) => panic!("Invalid index"),
             None => panic!("Out of string bounds"),
         }
    }

    //returns a reference to the node containing the k-bucket for the id
    pub fn lookup(&self, id: Key, mut index: usize, mut chunk: usize) -> Option<&Bucket> {
        if chunk == 31 && index == 7 {
            println!("{:?}",self.bucket);
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
                         println!("{:?}",self.bucket);
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
                        println!("{:?}",self.bucket);
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