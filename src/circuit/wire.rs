use bitcoin::hashes::{sha256, Hash};
use bitcoincore_rpc::bitcoin::key::rand::{self, Rng};

pub type HashValue = [u8; 32];
pub type PreimageValue = [u8; 32];

pub struct Preimages {
    pub zero: Option<PreimageValue>,
    pub one: Option<PreimageValue>,
}

pub struct HashTuple {
    pub zero: HashValue,
    pub one: HashValue,
}

pub struct Wire {
    pub preimages: Option<Preimages>,
    pub hashes: HashTuple,
    pub index: Option<usize>,
}

impl Wire {
    pub fn new(index: usize) -> Self {
        let mut rng = rand::thread_rng();

        let preimage1: PreimageValue = rng.gen();
        let preimage2: PreimageValue = rng.gen();

        let hash1 = sha256::Hash::hash(&preimage1).to_byte_array();
        let hash2 = sha256::Hash::hash(&preimage2).to_byte_array();

        Wire {
            preimages: Some(Preimages {
                zero: Some(preimage1),
                one: Some(preimage2),
            }),
            hashes: HashTuple {
                zero: hash1,
                one: hash2,
            },
            index: Some(index),
        }
    }
}
