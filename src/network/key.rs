use digest::Digest;
use primitive_types::H256;
use sha2::Sha256;
use to_binary::BinaryString;

#[derive(Debug, Hash, Eq, Clone, Ord, Copy, PartialEq, PartialOrd)]
pub struct Key(H256);

impl Key {
    /* Generates a random id to use as the kademlia ID */
    pub fn new(s: String) -> Key {
        let mut hasher = Sha256::new();
        hasher.update(&s);
        let mut k: H256 = H256::random();
        H256::assign_from_slice(&mut k, hasher.finalize().as_slice());
        Key(k)
    }

    #[inline]
    pub fn dist(self, other_key: Key) -> H256 {
        self.0 ^ other_key.0
    }

    pub fn as_bytes(&self) -> &[u8]{
        self.0.as_bytes()
    }
}

