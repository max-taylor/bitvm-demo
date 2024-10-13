use std::borrow::{Borrow, BorrowMut};

use bitcoin::{
    sighash::SighashCache, taproot::LeafVersion, TapLeafHash, Transaction, TxOut, XOnlyPublicKey,
};

use crate::actor::Actor;

use super::{generate_2_of_2_script, generate_challenge_script};

pub fn populate_challenge_tx_with_witness_data(
    verifier: &Actor,
    prover_pk: XOnlyPublicKey,
    challenge_tx: &mut Transaction,
    prevouts: &Vec<TxOut>,
    i: u64,
) {
    let mut sighash_cache = SighashCache::new(challenge_tx.borrow_mut());

    let sig_hash = sighash_cache
        .taproot_script_spend_signature_hash(
            1_usize,
            &bitcoin::sighash::Prevouts::All(&prevouts),
            TapLeafHash::from_script(
                &generate_2_of_2_script(prover_pk, verifier.pk),
                LeafVersion::TapScript,
            ),
            bitcoin::sighash::TapSighashType::Default,
        )
        .unwrap();
    // let mut challenge_tx = build_challenge_tx(
    //     &previous_response_tx,
    //     &challenge_address,
    //     &equivocation_address,
    //     amount,
    //     fee,
    //     dust_limit,
    //     i,
    // );

    // Sign the challenge transaction with the prover's key
    // let sighash =
    //     challenge_tx.signature_hash(i, &prover.x_only_public_key, bitcoin::SigHashType::ALL);
    //
    // let signature = prover.secp.sign(&sighash, &prover.keypair.secret_key);
    //
    // challenge_tx.input[i].witness = vec![signature.serialize_der().to_vec()];
}

pub fn populate_response_tx_with_witness_data(
    prover: &Actor,
    verifier: &Actor,
    response_tx: &mut Transaction,
    challenge_tx: &Transaction,
    challenge_hash: &[u8; 32],
    i: u64,
) {
    // let mut sighash_cache = SighashCache::new(response_tx.borrow_mut());
    //
    // let sig_hash = sighash_cache
    //     .taproot_script_spend_signature_hash(
    //         1_usize,
    //         &bitcoin::sighash::Prevouts::All(&challenge_tx.output),
    //         TapLeafHash::from_script(
    //             &generate_2_of_2_script(prover.pk, verifier.pk),
    //             LeafVersion::TapScript,
    //         ),
    //         bitcoin::sighash::TapSighashType::Default,
    //     )
    //     .unwrap();
    //
    // let musig_presigned_by_prover = prover.sign(sig_hash);
    // // println!("challenge sig: {:?}", challenge_sig);
    // // Verify needs to verify the signature
    // let mut sighash_cache = SighashCache::new(response_tx.borrow_mut());
    //
    // // Now we need to sign with the challenge_gate_num preimage to reveal a challenge
    // let challenge_script = generate_challenge_script(verifier.pk, challenge_hash);
    // let musig_2of2_script = generate_2_of_2_script(prover.pk, verifier.pk);
    //
    // let sig_hash = sighash_cache
    //     .taproot_script_spend_signature_hash(
    //         0,
    //         &bitcoin::sighash::Prevouts::All(&challenge_tx.output),
    //         TapLeafHash::from_script(&challenge_script, LeafVersion::TapScript),
    //         bitcoin::sighash::TapSighashType::Default,
    //     )
    //     .unwrap();
    // let verifier_challenge_sig = verifier.sign(sig_hash);
    //
    // let sig_hash = sighash_cache
    //     .taproot_script_spend_signature_hash(
    //         1,
    //         &bitcoin::sighash::Prevouts::All(&challenge_tx.output),
    //         TapLeafHash::from_script(&musig_2of2_script, LeafVersion::TapScript),
    //         bitcoin::sighash::TapSighashType::Default,
    //     )
    //     .unwrap();
    // let verifier_2of2_sig = verifier.sign(sig_hash);
    // let challenge_preimage = verifier.get_challenge_preimage(i as usize, challenge_gate_num)  // let challenge_control_block = challenge_taproot_info
    //     .control_block(&(challenge_script.clone(), LeafVersion::TapScript))
    //     .expect("Cannot create control block");
    //
    // let musig_control_block = equivocation_taproot_info
    //     .control_block(&(musig_2of2_script.clone(), LeafVersion::TapScript))
    //     .expect("Cannot create control block");
    //
    // let witness0 = sighash_cache.witness_mut(0).unwrap();
    // witness0.push(verifier_challenge_sig.as_ref());
    // witness0.push(challenge_preimage);
    // witness0.push(challenge_script);
    // witness0.push(&challenge_control_block.serialize());
    //
    // let witness1 = sighash_cache.witness_mut(1).unwrap();
    // witness1.push(verifier_2of2_sig.as_ref());
    // witness1.push(musig_presigned_by_prover.as_ref());
    // witness1.push(musig_2of2_script);
    // witness1.push(&musig_control_block.serialize());

    // let mut response_tx = build_response_tx(
    //     &challenge_tx,
    //     &response_address,
    //     &response_second_address,
    //     amount,
    //     fee,
    //     dust_limit,
    //     i,
    // );

    // Sign the response transaction with the prover's key
    // let sighash = response_tx.signature_hash(i, &prover.x_only_public_key, bitcoin::SigHashType::ALL);
    //
    // let signature = prover.secp.sign(&sighash, &prover.keypair.secret_key);
    //
    // response_tx.input[i].witness = vec![signature.serialize_der().to_vec()];
}
