use bitcoin::{
    hashes::{sha256, Hash},
    key::rand::{rngs::StdRng, SeedableRng},
};
use bitcoincore_rpc::bitcoin::key::rand::{self, Rng};
use serde::{Deserialize, Serialize};

pub type HashValue = [u8; 32];
pub type PreimageValue = [u8; 32];

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct HashTuple {
    pub zero: HashValue,
    pub one: HashValue,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct PreimageTuple {
    pub zero: Option<PreimageValue>,
    pub one: Option<PreimageValue>,
}

#[derive(Clone)]
pub struct Wire {
    pub preimages: Option<PreimageTuple>,
    pub hashes: HashTuple,
    pub index: Option<usize>,
    pub selector: Option<bool>,
}

impl Wire {
    pub fn new(index: usize, seed: Option<u64>) -> Self {
        let mut rng = match seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };

        let preimage1: PreimageValue = rng.gen();
        let preimage2: PreimageValue = rng.gen();

        let hash1 = sha256::Hash::hash(&preimage1).to_byte_array();
        let hash2 = sha256::Hash::hash(&preimage2).to_byte_array();

        Wire {
            preimages: Some(PreimageTuple {
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

    pub fn add_preimage(&mut self, preimage: PreimageValue) -> Option<Wire> {
        let hash = sha256::Hash::hash(&preimage).to_byte_array();
        if hash == self.hashes.zero {
            self.preimages = Some(PreimageTuple {
                zero: Some(preimage),
                one: match self.preimages {
                    Some(cur) => cur.one,
                    None => None,
                },
            });
        } else if hash == self.hashes.one {
            self.preimages = Some(PreimageTuple {
                zero: match self.preimages {
                    Some(cur) => cur.zero,
                    None => None,
                },
                one: Some(preimage),
            });
        } else {
            panic!("preimage does not match either hash");
        }
        if self.preimages.unwrap().zero.is_some() && self.preimages.unwrap().one.is_some() {
            return Some(self.clone());
        }
        None
    }
}
