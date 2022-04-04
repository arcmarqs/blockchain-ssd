extern crate mac_address;

use rand::Rng;
use primitive_types::U256;
use mac_address::get_mac_address;

#[derive(Hash,Eq,Clone,Ord,Copy,PartialEq,PartialOrd)]
pub struct Key(U256);

impl Key {
    /* Generates a random id to use as the kademlia ID */
    pub fn random() -> Key {
        let mut g = rand::thread_rng();
        let k: U256 = g.gen();
       
        Key(k)
    }
}

