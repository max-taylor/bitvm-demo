use bitcoin::hashes::Hash;
use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::taproot::TaprootSpendInfo;
use bitcoin::{sighash::SighashCache, taproot::LeafVersion, TapLeafHash};
use bitcoin::{Transaction, XOnlyPublicKey};

use crate::transactions::generate_2_of_2_script;
use crate::{
    actor::Actor,
    circuit::wire::{HashValue, PreimageValue},
    transactions::generate_challenge_script,
};

/**
* This function is called by the verifier to fill the response transaction with the witness data
**/
pub fn fill_response_tx_with_witness_data(
    response_tx: &mut Transaction,
    challenge_tx: &Transaction,
    verifier: &Actor,
    prover_pk: XOnlyPublicKey,
    challenge_hash: &HashValue,
    challenge_preimage: &PreimageValue,
    challenge_taproot_info: &TaprootSpendInfo,
    equivocation_taproot_info: &TaprootSpendInfo,
    prover_musig: &Signature,
    verifier_musig: &Signature,
) {
    let challenge_script = generate_challenge_script(verifier.pk, challenge_hash);

    let mut sighash_cache = SighashCache::new(response_tx);

    let sig_hash = sighash_cache
        .taproot_script_spend_signature_hash(
            0,
            &bitcoin::sighash::Prevouts::All(&challenge_tx.output),
            TapLeafHash::from_script(&challenge_script, LeafVersion::TapScript),
            bitcoin::sighash::TapSighashType::Default,
        )
        .unwrap();

    let verifier_challenge_sig = verifier.sign_tx(&sig_hash.to_byte_array());

    // FIX: This is failing with "cannot create control block"
    let challenge_control_block = challenge_taproot_info
        .control_block(&(challenge_script.clone(), LeafVersion::TapScript))
        .expect("Cannot create control block");

    let musig_2of2_script = generate_2_of_2_script(prover_pk, verifier.pk);

    let musig_control_block = equivocation_taproot_info
        .control_block(&(musig_2of2_script.clone(), LeafVersion::TapScript))
        .expect("Cannot create control block");

    let witness0 = sighash_cache.witness_mut(0).unwrap();
    witness0.push(verifier_challenge_sig.as_ref());
    witness0.push(challenge_preimage);
    witness0.push(challenge_script);
    witness0.push(&challenge_control_block.serialize());

    let witness1 = sighash_cache.witness_mut(1).unwrap();
    witness1.push(verifier_musig.as_ref());
    witness1.push(prover_musig.as_ref());
    witness1.push(musig_2of2_script);
    witness1.push(&musig_control_block.serialize());
}
