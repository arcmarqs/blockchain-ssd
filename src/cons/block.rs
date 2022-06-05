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
   pub timestamp: i64,
   pub data: Data   
}

#[derive(Debug)]
pub struct Data {
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
            hash = hash(data),
            timestamp = Gmt::now().timestamp(),
            Data {
                buyer = "null",
                seller = "null",
                amount = 0
            } 
        };
    }

    fn add_block(&mut self, block: Block) {
        let last_block = self.blocks.last();
        if(validate_block(&block, last_block) == true){
            self.block.push(block);
        } else {
            print!("invalid block!");
        }
    }

    fn validate_block(block: &Block, last_block: Block) {
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

    fn validate_chain(&self, chain: &[Block]) -> bool { 
        for i in 1..chain.len() {                                               //verificar se começa no 1
            let first = chain.get(i-1).expect("has to exist");
            let second = chain.get(i).expect("has to exist");
            if valid_block(second, first) != true {
                return false;
            } 
        }
        true
    }

    fn choose_chain (&mut self, local: Vec, remote: Vec) -> Vec {
        let is_local_valid = self.validate_chain(&local);       // não preciso de testar a local
        let is_remote_valid = self.validate_chain(&remote);

        if is_local_valid && is_remote_valid {
            if local.len() >= remote.len() {
                local
            }
            else {
                remote
            }
        }

        else if is_local_valid && !is_remote_valid {
            local
        }

        else if !is_local_valid && is_remote_valid {                   // nao preciso de testar a local
            remote
        }
    }


}

impl Block {
    pub fn new_block(id: u64, previous_hash: Vec<u8>, data: Data) -> Self {
        let (nonce, hash) = proof_of_work(previous_hash, data, id);
        let timestamp = Gmt::now().timestamp();
        Self { id,
            nonce, 
            prev_hash,                                                          // VER OS TIPOS DA HASH (STRING OU VEC<U8>)
            hash, 
            timestamp, 
            data}
    }
    
}

    pub fn proof_of_work(previous_hash: Vec<u8>, data: Data, id: u64) -> (u64, Vec<u8>) {
        let mut nonce_ex: u64;
    
        // Hash of our new block
        let mut hasher = Sha256::new();
        let buyer = Data.buyer;
        let seller = Data.seller;
        let ammout = Data.ammount;
        hasher.update(id.to_be_bytes());
        hasher.update(buyer.as_bytes());
        hasher.update(seller.as_bytes());
        hasher.update(ammount.to_bits().to_be_bytes());
        hasher.update(previous_hash);
        
        let final_hash = hasher.finalize();
        let final_hash_vec = final_hash.to_vec();
      
            
        // Descover the right nonce
        loop {
            let mut rng = thread_rng();
            nonce_ex = rng.gen();
            let nonce_bytes = nonce_ex.clone().to_be_bytes();
            let mut hasher1 = Sha256::new();
    
            hasher1.update(final_hash);
            hasher1.update(&nonce_bytes);
            let hash_with_xzeros = hasher1.finalize();
            
            
    
            if leading_zeros(&fcup) >= 8 {
                println!("Nonce: {} Final_Hash: {:x} Hash_dos_zeros {:x}", nonce_ex, final_hash, hash_with_xzeros);
    
    
                return (nonce_ex, final_hash_vec);
            }
        }
            
    }
    


fn test_proof_of_work(block: &Block) {    
    let nonce_bytes = block.nonce.close().to_be_bytes();
    hasher.update(&block.hash.as_bytes());
    hasher.update(&nonce_bytes);

    if (leading_zeros(&hasher.finalized()) >= 8) {
        return true;
    }
    false

}

pub fn proof_of_work(previous_hash: Vec<u8>, block: Block) -> (u64, Vec<u8>) {
    let mut nonce_ex: u64;

    // Hash of our new block
    let mut hasher = Sha256::new();
    let buyer = block.Data.buyer;
    let seller = block.Data.seller;
    let ammout = block.Data.ammount;
    hasher.update(buyer.as_bytes());
    hasher.update(seller.as_bytes());
    hasher.update(ammount.to_bits().to_be_bytes());
    hasher.update(previous_hash);
    
    let final_hash = hasher.finalize();
    let final_hash_vec = final_hash.to_vec();
  
        
    // Descover the right nonce
    loop {
        let mut rng = thread_rng();
        nonce_ex = rng.gen();
        let nonce_bytes = nonce_ex.clone().to_be_bytes();
        let mut hasher1 = Sha256::new();

        hasher1.update(final_hash);
        hasher1.update(&nonce_bytes);
        let hash_with_xzeros = hasher1.finalize();
        
        

        if leading_zeros(&fcup) >= 8 {
            println!("Nonce: {} Final_Hash: {:x} Hash_dos_zeros {:x}", nonce_ex, final_hash, hash_with_xzeros);


            return (nonce_ex, final_hash_vec);
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