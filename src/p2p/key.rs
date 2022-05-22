use digest::Digest;
use primitive_types::H256;
use sha2::Sha256;
use to_binary::BinaryString;

#[derive(Debug,Default, Hash, Eq, Clone, Ord, Copy, PartialEq, PartialOrd)]
pub struct Key(H256);

impl Key {
    /* Generates a random id to use as the kademlia ID */
    pub fn new(s: String) -> Key {
        let mut hasher = Sha256::new();
        hasher.update(&s);
        let mut k: H256 = H256::zero();
        H256::assign_from_slice(&mut k, hasher.finalize().as_slice());
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

