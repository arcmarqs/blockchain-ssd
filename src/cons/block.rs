use chrono::prelude::*;
use sha2::Sha256;
use primitive_types::U256;
use random::prelude::*;

#[derive(Debug)]
pub struct Block {
   pub id: u64,
   pub nonce: u64,
   pub prev_hash: U256,
   pub hash: U256,
   pub Data: String    
}

#[derive(Debug)]
pub struct Data {
    pub timestamp: i64,
    pub buyer: String,
    pub seller: String,
    pub amount: f64 
}

#[derive(Debug)]
pub struct Chain {
    pub blocks: Vec<Block>
}

fn hash(val: T) -> U256 {
    let mut hasher = Sha256::new();
    hasher.update(val);
    let result = hasher.finalize();
}

impl Chain {
    fn new() -> Self {
        Self {block: vec![]}
    }

    fn start(&mut self) {
        let mut hasher = Sha256::new();
        let data_hash = String::from("First block");
        let genesis = Block {
            id = 0,
            nonce = rand::random::<u64>(),
            prev_hash = 0,
            hash = hash(data)
            let data = DATA {} //ajuda amadeu
        };
    }

    fn add_block(&mut self, block: Block) {
        let last_block = self.blocks.last();
        if(validate(&block, last_block)){
            self.block.push(block);
        } else {
            print!("invalid block!");
        }
    }

    fn validate(block: &Block, last_block: Block) {
        if block.prev_hash != last_block.hash {
            warn!("block with id: {} has invalid prev_hash", block.id);
            return false;
        }

        if test_proof_of_work(block: &Block) {
            warn!("block with id: {} is a malicious block (wrong nonce)", block.id);
            return false;
        }
        true
    }


}

fn test_proof_of_work(block: &Block) {    
    let nonce_bytes = block.nonce.close().to_be_bytes();
    hasher.update(&block.hash.as_bytes());
    hasher.update(&nonce_bytes);

    if (leading_zeros(&hasher.finish()) == 8) {
        return true;
    }
    false

}

fn proof_of_work(previous_hash: Sha256) {
    let mut nonce: u64;
    loop {
        let mut hasher = Sha256::new();
        nonce = rng.gen();
        let nonce_bytes = nonce.clone().to_be_bytes();
        hasher.update(&node_id.as_bytes());
        hasher.update(&nonce_bytes);

        if leading_zeros(&hasher.finish()) == 8 {
            return true;
        }
    }
}

pub fn leading_zeros(bytes: &[u8]) -> u32 {
    let mut zeros = 0;
    for byte in bytes {
        let x = byte.leading_zeros();
        zeros += x;
        if x != 8 {
            break;
        }
    }
    zeros
}