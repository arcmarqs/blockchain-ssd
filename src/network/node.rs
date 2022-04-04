
use primitive_types::{H256,U256};

pub struct KadNode {
    kId: H256,
    ID: String,  // string version of kId, for readability
    IP: String,
    port: U256
}