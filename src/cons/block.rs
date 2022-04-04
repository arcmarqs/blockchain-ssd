use chrono::prelude::*;
use sha2::Sha256;
use primitive_types::H256;
use random::prelude::*;

pub struct Block {
   pub id: u64,
   pub timestamp: i64,
   pub nonce: u64,
   pub prev_hash: H256,
   pub hash: H256,
   pub Data: String    
}

pub struct Chain {
    pub blocks: Vec<Block>
}

fn hash(val: T) -> H256 {
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
        let genesis = Block {
            id = 0,
            timestamp = Gmt::now().timestamp(),
            nonce = rand::random::<u64>(),
            prev_hash = 0,
            data = String::from("First block"),
            hash = hash(data)
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
}