use std::{fmt, fs::{File, self}, io::{BufReader, Read}};

use openssl::{pkey::{Public, Private}, rsa::{Rsa, Padding}, sha::Sha256, symm::Cipher};
use primitive_types::H256;
use rand::Rng;

use super::{C1, C2};

#[derive(Clone)]
pub struct NodeValidator {
    node_id: NodeID,
    pub_key: Rsa<Public>,
    nonce: u64,
    priv_key: Rsa<Private>,
}
    
impl NodeValidator {
    /* Generates a random id to use as the kademlia ID */
    pub fn new() -> NodeValidator {
        let (k,pub_key,priv_key) = get_keypair();
        let node_id = NodeID(k);
        let nonce = solve_puzzle(node_id);     
        println!("k: {:?}", k.as_bytes());
        println!("NONCE: {:?}", nonce);

        NodeValidator {
            node_id: node_id,
            pub_key: Rsa::public_key_from_pem(&pub_key).unwrap(),
            nonce: nonce,
            priv_key: Rsa::private_key_from_pem_passphrase(&priv_key, " ".as_bytes()).unwrap(),

        }   
    }

    pub fn decrypt(&self, encrypted: &[u8]) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![0; self.priv_key.size() as usize];
        self.priv_key.private_decrypt(encrypted, &mut buf, Padding::PKCS1).unwrap();
        buf
    }

    pub fn get_pubkey(&self) -> Vec<u8> {
        self.pub_key.public_key_to_pem().unwrap()
    }

    pub fn get_nonce(&self) -> u64 {
        self.nonce
    }

    pub fn get_nodeid(&self) -> NodeID{
        self.node_id.clone()
    }

}

impl fmt::Debug for NodeValidator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeValidator")
         .field("nonce",&self.nonce)
         .field("pub_key", &self.pub_key)
         .field("priv_key", &self.priv_key)
         .finish()
    }
}
#[derive(Debug, Default, Hash, Eq, Clone, Ord, Copy, PartialEq, PartialOrd)]
pub struct NodeID(H256);

impl NodeID {

    #[inline]
    pub fn distance(self, other_key: NodeID) -> H256 {
        self.0 ^ other_key.0
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8]{
        self.0.as_bytes()
    }

    #[inline]
    pub fn from_vec(source: Vec<u8>) -> NodeID {
        NodeID(H256::from_slice(source.as_slice()))
    }

    // XOR's the key's first 8*chunk + index bits with a mask of size 8*chunk + index with the LSB set to 1
    pub fn set_bitmask(&self, index: usize, chunk: usize) -> NodeID {
        let mut bitmask: [u8;32] = [0;32];
        bitmask[chunk] = u8::pow(2,7-index as u32);
        let my_key = H256::from_slice(&bitmask);

        NodeID(self.0 ^ my_key)
    }

    // retunrs a slice containing the key up to 8*chunk + index
    pub fn prefix(&self, index: usize, chunk: usize) -> Vec<u8> {
        let mut prefix: Vec<u8> = self.0.as_bytes()[0..chunk+1].to_owned();

        if index == 0 {
            if prefix[chunk] > 127 {
                return vec![1];
            } else {
                return vec![0];
            }
        }
        let range = 255 >> (8 - index);
        prefix[chunk] = prefix[chunk] | !range;
        prefix
    }
}

fn solve_puzzle(node_id: NodeID) -> u64 {
    let mut rng = rand::thread_rng();
    let mut nonce: u64;
    loop {
        let mut hasher = Sha256::new();
        nonce = rng.gen();
        let nonce_bytes = nonce.clone().to_be_bytes();
        hasher.update(node_id.as_bytes());
        hasher.update(&nonce_bytes);

        if leading_zeros(&hasher.finish()) >= C2 {
            return nonce;
        }
    }
}

pub fn verify_puzzle(node_id: NodeID, nonce: u64) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(node_id.as_bytes());
    hasher.update(&nonce.to_be_bytes());

    if leading_zeros(&hasher.finish()) >= C2 {
        return true;
    }

    false
}

fn get_keypair() -> (H256,Vec<u8>,Vec<u8>) {
    let pub_location = "config/pub_key";
    let priv_location = "config/priv_key";
    match (File::open(&pub_location),File::open(&priv_location)) {
        (Ok(fpub),Ok(fpriv)) => {
            let mut hasher = Sha256::new();
            let mut pubreader = BufReader::new(fpub);
            let mut pubk: String = String::new();
            let mut privreader = BufReader::new(fpriv);
            let mut privk: String = String::new();
            pubreader.read_to_string(&mut pubk).unwrap();
            privreader.read_to_string(&mut privk).unwrap();
            hasher.update(pubk.as_bytes());
            let id_key = hasher.finish();
            hasher = Sha256::new();
            hasher.update(&id_key);
            if leading_zeros(&hasher.finish()) >= C1 {
                //if the node id doesn't solve the static puzzle we create a new pair
                return gen_keypair(pub_location,priv_location);
            }
            (H256::from_slice(&id_key),pubk.as_bytes().to_owned(),privk.as_bytes().to_owned())
        },  
        _ => gen_keypair(pub_location,priv_location),
    }
}

fn gen_keypair(pub_location: &str, priv_location: &str) -> (H256,Vec<u8>,Vec<u8>) {
    let passphrase = " ";
    let mut private_key: Vec<u8>;
    let mut public_key: Vec<u8>;
    let mut hashed_key;
    loop {
        let mut hasher = Sha256::new();
        let rsa = Rsa::generate(2048).unwrap();
        private_key = rsa.private_key_to_pem_passphrase(Cipher::aes_128_cbc(), passphrase.as_bytes()).unwrap();
        public_key= rsa.public_key_to_pem().unwrap();
        hasher.update(&public_key);
        hashed_key = hasher.finish();
        hasher = Sha256::new();
        hasher.update(&hashed_key);
        if leading_zeros(&hasher.finish()) >= C1 {
            break;
        }
    }
    fs::write(pub_location,String::from_utf8(public_key.clone()).unwrap()).unwrap();
    fs::write(priv_location, String::from_utf8(private_key.clone()).unwrap()).unwrap();
    let id = H256::from_slice(&hashed_key);
    (id,public_key,private_key)
}


pub fn leading_zeros(bytes: &[u8]) -> u32{
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

pub fn encrypt_message(publ_key: &[u8], message: &[u8]) -> Vec<u8> {
    let pub_key = Rsa::public_key_from_pem(publ_key).unwrap();
    let mut buf: Vec<u8> = vec![0; pub_key.size() as usize];
    pub_key.public_encrypt(message, &mut buf, Padding::PKCS1).unwrap();

    buf
}
