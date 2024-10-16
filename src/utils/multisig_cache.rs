use bitcoin::{
    hashes::Hash,
    key::Secp256k1,
    secp256k1::{schnorr::Signature, All, Message},
    sighash::SighashCache,
    taproot::LeafVersion,
    TapLeafHash, TapSighash, Transaction, TxOut, XOnlyPublicKey,
};

use crate::{actor::ActorType, transactions::generate_2_of_2_script};

pub struct MultiSigCache {
    pub signatures: Vec<Signature>,
    prover_pk: Option<XOnlyPublicKey>,
    verifier_pk: Option<XOnlyPublicKey>,
    pub actor_type: ActorType,
}

impl MultiSigCache {
    pub fn new(actor_type: ActorType, actor_pk: XOnlyPublicKey) -> MultiSigCache {
        let mut cache = MultiSigCache {
            actor_type,
            prover_pk: None,
            verifier_pk: None,
            signatures: Vec::new(),
        };

        match actor_type {
            ActorType::Prover => {
                cache.prover_pk = Some(actor_pk);
            }
            ActorType::Verifier => {
                cache.verifier_pk = Some(actor_pk);
            }
        }

        cache
    }

    pub fn set_other_actor_pk(&mut self, actor_pk: XOnlyPublicKey) {
        match self.actor_type {
            ActorType::Prover => {
                self.verifier_pk = Some(actor_pk);
            }
            ActorType::Verifier => {
                self.prover_pk = Some(actor_pk);
            }
        }
    }

    pub fn add_signature(
        &mut self,
        secp: &Secp256k1<All>,
        signature: Signature,
        tx: &Transaction,
        last_output: Vec<TxOut>,
    ) {
        let prover_pk = self.prover_pk.unwrap();
        let verifier_pk = self.verifier_pk.unwrap();

        let sig_hash = get_sighash_for_musig_script(tx, &last_output, prover_pk, verifier_pk);

        let pubkey = match self.actor_type {
            ActorType::Prover => verifier_pk,
            ActorType::Verifier => prover_pk,
        };

        secp.verify_schnorr(
            &signature,
            &Message::from_digest_slice(sig_hash.as_byte_array()).expect("should be hash"),
            &pubkey,
        )
        .unwrap();
        self.signatures.push(signature);
    }

    pub fn get_prover_pk(&self) -> XOnlyPublicKey {
        self.prover_pk.unwrap()
    }

    pub fn get_verifier_pk(&self) -> XOnlyPublicKey {
        self.verifier_pk.unwrap()
    }

    pub fn get_signatures(&self) -> Vec<Signature> {
        self.signatures.clone()
    }

    pub fn get_signature(&self, index: usize) -> Signature {
        self.signatures[index]
    }
}

/**
* Generates the sighash for a transaction containing a musig script in vout 1
**/
pub fn get_sighash_for_musig_script(
    tx: &Transaction,
    last_output: &Vec<TxOut>,
    prover_pk: XOnlyPublicKey,
    verifier_pk: XOnlyPublicKey,
) -> TapSighash {
    let mut sighash_cache = SighashCache::new(tx);

    sighash_cache
        .taproot_script_spend_signature_hash(
            1_usize,
            &bitcoin::sighash::Prevouts::All(&last_output),
            TapLeafHash::from_script(
                &generate_2_of_2_script(prover_pk, verifier_pk),
                LeafVersion::TapScript,
            ),
            bitcoin::sighash::TapSighashType::Default,
        )
        .unwrap()
}
