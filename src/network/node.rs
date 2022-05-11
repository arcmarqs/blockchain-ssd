
use primitive_types::{H256,U256};
use crate::key::{Key};

#[derive(Debug,Clone,Eq,Ord,PartialEq,PartialOrd)]
pub struct KadNode {
    kId: Key,
    id_string: String,  // string version of kId, for readability
    ip: String,
    port: u8
}
