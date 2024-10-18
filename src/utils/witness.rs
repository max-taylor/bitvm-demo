use bitcoin::hashes::Hash;
use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::taproot::TaprootSpendInfo;
use bitcoin::{sighash::SighashCache, taproot::LeafVersion, TapLeafHash};
use bitcoin::{Transaction, XOnlyPublicKey};

use crate::circuit::wire::{HashTuple, PreimageTuple};
use crate::circuit::BristolCircuit;
use crate::transactions::{generate_2_of_2_script, generate_anti_contradiction_script};
use crate::{actor::Actor, transactions::generate_challenge_script};

use super::challenge_hashes::ChallengeHashesManager;

/**
* This function is called by the verifier to fill the response transaction with the witness data
**/
pub fn fill_response_tx_with_witness_for_gate_challenge(
    response_tx: &mut Transaction,
    challenge_tx: &Transaction,
    verifier: &Actor,
    prover_pk: XOnlyPublicKey,
    challenge_hash_manager: &ChallengeHashesManager,
    challenge_response_index: usize,
    gate_to_challenge: usize,
    challenge_taproot_info: &TaprootSpendInfo,
    equivocation_taproot_info: &TaprootSpendInfo,
    prover_musig: &Signature,
    verifier_musig: &Signature,
) {
    let challenge_hashes = challenge_hash_manager.get_challenge_hashes(challenge_response_index);

    let challenge_script =
        generate_challenge_script(verifier.pk, &challenge_hashes[gate_to_challenge]);

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
        .expect("Cannot create challenge control block");

    // Challenge witness data
    let witness0 = sighash_cache.witness_mut(0).unwrap();
    witness0.push(verifier_challenge_sig.as_ref());
    witness0.push(
        challenge_hash_manager.get_challenge_preimage(challenge_response_index, gate_to_challenge),
    );
    witness0.push(challenge_script);
    witness0.push(&challenge_control_block.serialize());

    let musig_2of2_script = generate_2_of_2_script(prover_pk, verifier.pk);

    let musig_control_block = equivocation_taproot_info
        .control_block(&(musig_2of2_script.clone(), LeafVersion::TapScript))
        .expect("Cannot create equivocation control block");

    // Equivocation witness data
    let witness1 = sighash_cache.witness_mut(1).unwrap();
    witness1.push(verifier_musig.as_ref());
    witness1.push(prover_musig.as_ref());
    witness1.push(musig_2of2_script);
    witness1.push(&musig_control_block.serialize());
}

pub fn fill_response_tx_with_witness_for_equivocation(
    response_tx: &mut Transaction,
    challenge_tx: &Transaction,
    verifier: &Actor,
    equivocation_taproot_info: &TaprootSpendInfo,
    hashes: HashTuple,
    preimages: PreimageTuple,
) {
    let equivocation_script = generate_anti_contradiction_script(hashes, verifier.pk);
    let equivocation_control_block = equivocation_taproot_info
        .control_block(&(equivocation_script.clone(), LeafVersion::TapScript))
        .expect("Cannot create equivocation control block");

    let mut sighash_cache = SighashCache::new(response_tx);

    let sig_hash = sighash_cache
        .taproot_script_spend_signature_hash(
            0,
            &bitcoin::sighash::Prevouts::All(&[challenge_tx.output[1].clone()]),
            TapLeafHash::from_script(&equivocation_script, LeafVersion::TapScript),
            bitcoin::sighash::TapSighashType::Default,
        )
        .unwrap();

    let equivocation_sig = verifier.sign_tx(&sig_hash.to_byte_array());

    // Equivocation witness data
    let witness = sighash_cache.witness_mut(0).unwrap();
    witness.push(equivocation_sig.as_ref());
    witness.push(preimages.one.unwrap());
    witness.push(preimages.zero.unwrap());
    witness.push(equivocation_script);
    witness.push(&equivocation_control_block.serialize());
}
