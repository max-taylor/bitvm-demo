use bitcoin::{
    hashes::{sha256, Hash},
    key::rand::{self, Rng},
};

use crate::circuit::wire::{HashValue, PreimageValue};

pub struct ChallengeHashesManager {
    pub challenge_hashes: Vec<Vec<HashValue>>,
    pub challenge_preimages: Vec<Vec<PreimageValue>>,
}

impl ChallengeHashesManager {
    pub fn new() -> ChallengeHashesManager {
        ChallengeHashesManager {
            challenge_hashes: vec![],
            challenge_preimages: vec![],
        }
    }

    pub fn generate_challenge_hashes(
        &mut self,
        num_gates: usize,
    ) -> (Vec<HashValue>, Vec<PreimageValue>) {
        let mut challenge_hashes: Vec<HashValue> = Vec::new();
        let mut rng = rand::thread_rng();
        let mut preimages = Vec::new();
        for _ in 0..num_gates {
            let preimage: PreimageValue = rng.gen();
            preimages.push(preimage);
            challenge_hashes.push(sha256::Hash::hash(&preimage).to_byte_array());
        }
        self.challenge_preimages.push(preimages.clone());
        self.challenge_hashes.push(challenge_hashes.clone());
        (challenge_hashes, preimages)
    }

    pub fn add_challenge_hashes(&mut self, challenge_hashes: Vec<HashValue>) {
        self.challenge_hashes.push(challenge_hashes);
    }

    pub fn get_challenge_hashes(&self, index: usize) -> Vec<HashValue> {
        self.challenge_hashes[index].clone()
    }

    pub fn get_challenge_preimage(&self, index: usize, gate_num: usize) -> PreimageValue {
        self.challenge_preimages[index][gate_num]
    }
}
