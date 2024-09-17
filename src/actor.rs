use std::str::FromStr;

use bitcoin::{
    key::secp256k1::{Keypair, Secp256k1, SecretKey},
    secp256k1::{All, XOnlyPublicKey},
    Address,
};
use bitcoincore_rpc::bitcoin::key::rand::{self, Rng};

pub struct Actor {
    keypair: Keypair,
    pub address: Address,
}

impl Actor {
    pub fn new() -> Self {
        // Initialize the Secp256k1 context
        let secp: Secp256k1<All> = Secp256k1::new();

        // Generate random string
        let mut rng = rand::thread_rng();
        let random_string: String = rng.gen::<u8>().to_string();

        // Define the internal key
        let internal_secret = SecretKey::from_str(random_string.as_str()).unwrap();

        let keypair = Keypair::from_secret_key(&secp, &internal_secret);
        let (xonly, _parity) = XOnlyPublicKey::from_keypair(&keypair);

        let address = Address::p2tr(&secp, xonly, None, bitcoin::Network::Regtest);
        Actor { keypair, address }
    }
}
