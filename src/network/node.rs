use crate::kad::{KadNode};
use crate::key::{Key};

#[derive(Debug,Clone)]
pub struct Bucket(Vec<Box<KadNode>>);

impl Bucket {
    pub fn new(nodes: Vec<KadNode>) -> Bucket {
        Bucket(Vec::new())
    }

    pub fn insert(&mut self, node: Box<KadNode>) {
        self.0.push(node);
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