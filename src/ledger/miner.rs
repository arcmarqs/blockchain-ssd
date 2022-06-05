use parking_lot::{Mutex, RwLock};
use super::block::{Block, Chain, Data};

#[derive(Debug)]
pub struct Miner {
    transactions: Mutex<Vec<Data>>,
    blocks_to_validate: Mutex<Vec<Block>>,
    blockchain: RwLock<Chain>,
}

impl Miner {
    pub fn new() -> Miner {
        Miner {
            transactions: Mutex::new(Vec::new()),
            blocks_to_validate: Mutex::new(Vec::new()),
            blockchain: RwLock::new(Chain::new()),
        }
    }
    pub fn print_blockchain(&self) {
        let lock = self.blockchain.try_read().unwrap();
        println!("Blockchain: {:?}", lock.blocks)
    }

    pub fn get_chain(&self) -> Chain {
        self.blockchain.read().get_chain()
    }

    pub fn store_transaction(&self, t: Data) {
        let mut lock = self.transactions.lock();
        lock.push(t);
    }

    pub fn store_block(&self,block: Block) {
        let mut lock = self.blocks_to_validate.lock();
        lock.push(block);
    }

    pub fn mine(&self) -> Block {
       let transaction;
       {
           transaction = self.transactions.lock().pop();
       }
       match transaction {
            Some(t) => 
            {
                self.blockchain.write().mine(t)

            },
            None => todo!(),
        }
    }

    pub fn validate_blocks(&self) -> Result<(), &'static str> {
        let block;
        {
            block = self.blocks_to_validate.lock().pop();
        }
        match block {
             Some(b) => 
             {
                if self.blockchain.write().add_block(b){ 
                    Ok(())
                } else {
                    Err("Invalid block")
                }
 
             },
             None => Err("No block"),
         }
    }

    pub fn choose_chain(&self, mut chains: Vec<Vec<Block>>) {
        chains.sort_by(|a,b| b.len().cmp(&a.len()));

        self.blockchain.write().replace(chains.pop().unwrap());
    }
}
