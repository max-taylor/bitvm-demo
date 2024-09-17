use bitcoin::hashes::{sha256, Hash};
use bitcoincore_rpc::bitcoin::key::rand::{self, Rng};
use serde::{Deserialize, Serialize};

pub type HashValue = [u8; 32];
pub type PreimageValue = [u8; 32];

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Preimages {
    pub zero: Option<PreimageValue>,
    pub one: Option<PreimageValue>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct HashTuple {
    pub zero: HashValue,
    pub one: HashValue,
}

#[derive(Clone)]
pub struct Wire {
    pub preimages: Option<Preimages>,
    pub hashes: HashTuple,
    pub index: Option<usize>,
    pub selector: Option<bool>,
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
            selector: None,
        }
    }

    pub fn get_hash_pair(&self) -> HashTuple {
        self.hashes
    }

    pub fn get_preimage_of_selector(&self) -> [u8; 32] {
        match self.preimages {
            Some(preimage_tuple) => match self.selector {
                Some(b) => {
                    if !b {
                        preimage_tuple.zero.unwrap()
                    } else {
                        preimage_tuple.one.unwrap()
                    }
                }
                None => panic!("selector is not set"),
            },
            None => panic!("preimages are not set"),
        }
    }
}
