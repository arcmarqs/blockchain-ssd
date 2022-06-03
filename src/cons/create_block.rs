use openssl::sha::Sha256;
use rand::Rng;




pub fn proof_of_work(previous_hash: Vec<u8>, auction_id: u64) {
    let mut nonce_ex: u64;

    // Hash of our new block
    let mut hasher = Sha256::new();
    hasher.update(previous_hash.as_bytes());
    hasher.update(auction_id.as_bytes());
    let final_hash = hasher.finalize().to_vec();
        
    // Descover the right nonce
    loop {
        nonce_ex = rng.gen();
        let nonce_bytes = nonce_ex.clone().to_be_bytes();

        hasher.update(&final_hash.as_bytes());
        hasher.update(&nonce_bytes);

        if leading_zeros(&hasher.finish()) == 8 {
            return true;
            //println!("ACERTOU");
            //retornar a hash e a nonce 

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