use chrono::Utc;
use openssl::sha::Sha256;
use primitive_types::H256;
use rand::{random, Rng};

use crate::{p2p::key::{NodeID, leading_zeros}, auctions::auction::AuctionGossip};
const DIFFICULTY: u32 = 8;

#[derive(Debug,Clone)]
pub struct Block {
   pub id: u64,
   pub nonce: u64,
   pub prev_hash: H256,
   pub hash: H256,
   pub timestamp: i64,
   pub data: Data   
}

impl Block {
    pub fn new(id: u64,nonce : u64, prev_hash: H256, hash: H256, timestamp: i64,data: Data) -> Block {
        Block {
            id,
            nonce,
            prev_hash,
            hash,
            timestamp,
            data,
        }
    }

    pub fn mine_block(id: u64, previous_hash: H256, data: Data) -> Block {
        let (nonce, hash) = proof_of_work(previous_hash, &data);
        let timestamp = Utc::now().timestamp();
        Block{ 
            id,
            nonce, 
            prev_hash : previous_hash,                                                          // VER OS TIPOS DA HASH (STRING OU VEC<U8>)
            hash, 
            timestamp, 
            data
        }
    }
    
}

#[derive(Debug,Clone)]
pub struct Data {
    buyer: NodeID,
    seller: NodeID,
    amount: f32,
    auction_id: H256,
}

impl Data {
    pub fn new(buyer: NodeID, seller: NodeID, amount: f32, auction_id: H256) -> Data {
        Data {
            buyer,
            seller,
            amount,
            auction_id,
        }
    }

    pub fn hash(&self) -> H256 {
        let mut hasher = Sha256::new();
        hasher.update(self.buyer.as_bytes());
        hasher.update(self.seller.as_bytes());
        hasher.update(&self.amount.to_be_bytes());
        hasher.update(self.auction_id.as_bytes());

        H256::from(hasher.finish())
    }

    pub fn get_auction_id(&self) -> H256 {
        self.auction_id
    }

    pub fn get_amount(&self) -> f32 {
        self.amount
    }

    pub fn get_seller(&self) -> NodeID {
        self.seller
    }

    pub fn get_buyer(&self) -> NodeID {
        self.buyer
    }

    pub fn from_auction(auction: AuctionGossip) -> Data {
        Data {
            // should be buyer, just testing blockchain
            buyer: auction.get_seller(),
            seller: auction.get_seller(),
            amount: auction.get_price(),
            auction_id:auction.get_auction_id() ,
        }
    }
}

#[derive(Debug,Clone)]
pub struct Chain {
   pub blocks: Vec<Block>
}

impl Chain {
    pub fn new() -> Self {
        let null_node = NodeID::from_h256(H256::zero());
        let data = Data::new(null_node,null_node,0.0,H256::zero());
        let hash = data.hash();
        let (nonce,cur_hash) = proof_of_work(H256::zero(),&data);

        let genesis = Block {
            id : 0,
            nonce,
            prev_hash: H256::zero(),
            hash : cur_hash,
            timestamp : Utc::now().timestamp(),
            data,
        };

        Self {blocks: vec![genesis]}
    }

    pub fn get_chain(&self) -> Chain {
        self.clone()
    }

    pub fn replace(&mut self, chain: Vec<Block>) {
        self.blocks = chain;
    }

    pub fn mine(&mut self, data: Data) -> Block {
        let prev_block = self.blocks.last().unwrap();
        let block = Block::mine_block(prev_block.id, prev_block.hash,data);
        self.blocks.push(block.clone());
        return block;
    }

    pub fn add_block(&mut self, block: Block) -> bool{
        let last_block = self.blocks.last().unwrap();
        if self.validate_block(&block, last_block) {
            self.blocks.push(block);
            true
        } else {
            false
        }
    }

    fn validate_block(&self,block: &Block, last_block: &Block) -> bool {
        if block.prev_hash != last_block.hash {
            println!("block with id: {} has invalid prev_hash", block.id);
            return false;
        }

        else if block.id != (last_block.id + 1) {
            println!("block with id: {} has invalid id", block.id);
            return false;
        }

        else if test_proof_of_work(block) {
            println!("block with id: {} is a malicious block (wrong nonce)", block.id);
            return false;
        }
        true
    }

    pub fn validate_chain(&self,chain: &Chain) -> bool { 
        for i in 1..chain.blocks.len() {                                               //verificar se começa no 1
            let first = chain.blocks.get(i-1).expect("has to exist");
            let second = chain.blocks.get(i).expect("has to exist");
            if self.validate_block(second, first) != true {
                return false;
            } 
        }
        true
    }

    pub fn choose_chain(&self, local: Chain, remote: Chain) -> Chain {
        let is_local_valid = self.validate_chain(&local);       // não preciso de testar a local
        let is_remote_valid = self.validate_chain(&remote);

        match (is_local_valid,is_remote_valid) {
            (true, true) => {
                if local.blocks.len() >= remote.blocks.len() {
                    local
                } else {
                    remote
                }
            },
            (true, false) => local,
            (false, true) => remote,
            (false, false) => local, // this is not supposed to happen
        }
     
    }

    pub fn broadcast_block(&self, block: Block) {

    }
}


pub fn proof_of_work(previous_hash: H256, data: &Data) -> (u64, H256) {
    let mut nonce_ex: u64;

    // Hash of our new block
    let mut hasher = Sha256::new();
    let data_hash = data.hash();
    hasher.update(previous_hash.as_bytes());
    hasher.update(data_hash.as_bytes());
    let new_block_hash = H256::from(hasher.finish()); 
    // Descover the right nonce
    loop {
        let mut rng = rand::thread_rng();
        nonce_ex = rng.gen();
        let nonce_bytes = nonce_ex.clone().to_be_bytes();
        let mut hasher = Sha256::new();

        hasher.update(new_block_hash.as_bytes());
        hasher.update(&nonce_bytes);   

        if leading_zeros(&hasher.finish()) >= DIFFICULTY {    
            return (nonce_ex, new_block_hash);
        }
    }      
}
    
fn check_block_integrity(block : &Block) -> bool {
    block.data.hash() == block.hash
}

fn test_proof_of_work(block: &Block) -> bool {
    if  !(block.data.hash() == block.hash) {
        return false;
    }  

    let nonce_bytes = block.nonce.clone().to_be_bytes();
    let mut hasher = Sha256::new();
    hasher.update(&block.hash.as_bytes());
    hasher.update(&nonce_bytes);

    if leading_zeros(&hasher.finish()) >= DIFFICULTY {
        return true;
    }
    false
}

