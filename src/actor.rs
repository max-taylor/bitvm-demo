use std::str::FromStr;

use bitcoin::{
    address::NetworkChecked,
    hashes::Hash,
    key::{
        rand::RngCore,
        secp256k1::{Keypair, Secp256k1, SecretKey},
    },
    secp256k1::{schnorr::Signature, All, Message, XOnlyPublicKey},
    Address, TapNodeHash, TapSighash, TapTweakHash, Transaction, TxOut,
};
use bitcoincore_rpc::bitcoin::key::rand::{self};

use crate::utils::multisig_cache::{get_sighash_for_musig_script, MultiSigCache};

#[derive(Debug, Clone, Copy)]
pub enum ActorType {
    Prover,
    Verifier,
}

pub struct Actor {
    pub keypair: Keypair,
    pub address: Address<NetworkChecked>,
    pub secp: Secp256k1<All>,
    pub pk: XOnlyPublicKey,
    pub multisg_cache: MultiSigCache,
    pub actor_type: ActorType,
}

impl Actor {
    pub fn new(actor_type: ActorType) -> Self {
        // Initialize the Secp256k1 context
        let secp: Secp256k1<All> = Secp256k1::new();

        // Generate a random 32-byte array
        let mut rng = rand::thread_rng();
        let mut random_bytes = [0u8; 32];
        rng.fill_bytes(&mut random_bytes);

        // Convert the random bytes into a SecretKey
        let internal_secret = SecretKey::from_slice(&random_bytes).expect("Invalid secret key");

        // Create the keypair using the generated secret key
        let keypair = Keypair::from_secret_key(&secp, &internal_secret);
        let (xonly, _parity) = XOnlyPublicKey::from_keypair(&keypair);

        // Generate an address (p2tr in this case)
        let address = Address::p2tr(&secp, xonly, None, bitcoin::Network::Regtest);

        Actor {
            keypair,
            address,
            secp,
            pk: xonly,
            actor_type,
            multisg_cache: MultiSigCache::new(actor_type, xonly),
        }
    }

    pub fn add_signature(
        &mut self,
        signature: Signature,
        tx: &Transaction,
        last_output: Vec<TxOut>,
    ) {
        self.multisg_cache
            .add_signature(&self.secp, signature, tx, last_output)
    }

    pub fn get_bitcoincore_rpc_address(
        &self,
    ) -> bitcoincore_rpc::bitcoin::Address<bitcoincore_rpc::bitcoin::address::NetworkChecked> {
        bitcoincore_rpc::bitcoin::Address::from_str(self.address.to_string().as_str())
            .unwrap()
            .assume_checked()
    }

    pub fn sign_with_tweak(
        &self,
        sighash: TapSighash,
        merkle_root: Option<TapNodeHash>,
    ) -> Signature {
        self.secp.sign_schnorr_with_rng(
            &Message::from_digest_slice(sighash.as_byte_array()).expect("should be hash"),
            &self
                .keypair
                .add_xonly_tweak(
                    &self.secp,
                    &TapTweakHash::from_key_and_tweak(self.pk, merkle_root).to_scalar(),
                )
                .unwrap(),
            &mut rand::thread_rng(),
        )
    }

    pub fn sign_tx(&self, sighash_bytes: &[u8; 32]) -> Signature {
        self.secp.sign_schnorr_with_rng(
            &Message::from_digest_slice(sighash_bytes).expect("should be hash"),
            &self.keypair,
            &mut rand::thread_rng(),
        )
    }

    pub fn sign_tx_containing_musig(&self, tx: &Transaction, last_output: Vec<TxOut>) -> Signature {
        let prover_pk = self.multisg_cache.get_prover_pk();
        let verifier_pk = self.multisg_cache.get_verifier_pk();
        let sighash = get_sighash_for_musig_script(tx, &last_output, prover_pk, verifier_pk);
        self.sign_tx(&sighash.to_byte_array())
    }
}
