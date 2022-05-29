use digest::Digest;
use openssl::pkey::Public;
use openssl::symm::Cipher;
use primitive_types::H256;
use openssl::sha::Sha256;
use openssl::rsa::{Rsa,Padding};
use to_binary::BinaryString;
use std::fs::File;
use std::fs;
use std::io::{prelude::*, BufReader, BufWriter};


#[derive(Debug,Default, Hash, Eq, Clone, Ord, Copy, PartialEq, PartialOrd)]
pub struct Key(H256);

impl Key {
    /* Generates a random id to use as the kademlia ID */
    pub fn new() -> Key {
        let (pub_key,_priv_key) = get_keypair();
        let mut hasher = Sha256::new();
        hasher.update(&pub_key);
        let mut k = H256::zero();
        H256::assign_from_slice(&mut k, &hasher.finish());
        Key(k)
    }

    #[inline]
    pub fn distance(self, other_key: Key) -> H256 {
        self.0 ^ other_key.0
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8]{
        self.0.as_bytes()
    }

    #[inline]
    pub fn from_vec(source: Vec<u8>) -> Key {
        Key(H256::from_slice(source.as_slice()))
    }

    // XOR's the key's first 8*chunk + index bits with a mask of size 8*chunk + index with the LSB set to 1
    pub fn set_bitmask(&self, index: usize, chunk: usize) -> Key {
        let mut bitmask: [u8;32] = [0;32];
        bitmask[chunk] = u8::pow(2,7-index as u32);
        let my_key = H256::from_slice(&bitmask);

        Key(self.0 ^ my_key)
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

fn get_keypair() -> (Vec<u8>,Vec<u8>) {
    let pub_location = "config/pub_key";
    let priv_location = "config/priv_key";
    match (File::open(&pub_location),File::open(&priv_location)) {
        (Ok(fpub),Ok(fpriv)) => {
            let mut pubreader = BufReader::new(fpub);
            let mut pubk: String = String::new();
            let mut privreader = BufReader::new(fpriv);
            let mut privk: String = String::new();
            pubreader.read_to_string(&mut pubk).unwrap();
            privreader.read_to_string(&mut privk).unwrap();
            (pubk.as_bytes().to_owned(),privk.as_bytes().to_owned())
        },  
        _ =>{
            let passphrase = " ";
            let rsa = Rsa::generate(1024).unwrap();
            let private_key: Vec<u8> = rsa.private_key_to_pem_passphrase(Cipher::aes_128_cbc(), passphrase.as_bytes()).unwrap();
            let public_key: Vec<u8> = rsa.public_key_to_pem().unwrap();
            fs::write(pub_location,String::from_utf8(public_key.clone()).unwrap()).unwrap();
            fs::write(priv_location, String::from_utf8(private_key.clone()).unwrap()).unwrap();
            (public_key,private_key)
        },
    }
}
