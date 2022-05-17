
use crate::network::node::{Contact,Node,Bucket};
use crate::network::key::Key;
use crate::network::kad::{KadNode};
use to_binary::BinaryString;

#[derive(Debug,Clone)]
pub struct Rtable{pub head: Node}

impl Rtable {
    pub fn new() -> Rtable {
        Rtable {
        head : Node::new(),
        }
    }

    pub fn init(&mut self, origin: Contact) {
        self.head.init_tree(origin,0,0);
    }
    //inserts the contact in the appropriated kbucket.
    pub fn insert(&mut self, con: Contact) {
        self.head.insert(con,0,0);
    }

    pub fn lookup(&self, id: Key) -> &Node {
        self.head.lookup(id,0,0)
    }
}
