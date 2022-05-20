
use super::node::{Contact,Node,Bucket};
use super::kad::{KadNode};
use super::key::Key;
use to_binary::BinaryString;

#[derive(Debug,Default,Clone)]
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

    pub fn lookup(&self, id: Key) -> &Node {
        self.head.lookup(id,0,0)
    }
}
