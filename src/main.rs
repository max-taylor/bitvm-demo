use actor::{Actor, ActorType};
use bitcoin::{key::Secp256k1, sighash::SighashCache, Amount, Transaction, TxOut};
use bitcoincore_rpc::RpcApi;
use circuit::BristolCircuit;
use constants::WALLET_NAME;
use transactions::{
    challenge::{build_challenge_tx, build_response_tx},
    generate_2_of_2_script, generate_challenge_address_and_info,
    generate_equivocation_address_and_info, generate_response_address_and_info,
    generate_timelock_script, taproot_address_from_script_leaves,
};
use utils::{
    bitcoin_rpc::setup_client_and_fund_prover, challenge_hashes::ChallengeHashesManager,
    conversions::number_to_bool_array,
};

mod actor;
mod circuit;
mod constants;
mod traits;
mod transactions;
mod utils;

fn main() {
    let mut circuit = BristolCircuit::from_bristol("circuits/add.txt");

    let mut prover = Actor::new(ActorType::Prover);
    let mut verifier = Actor::new(ActorType::Verifier);

    prover.multisg_cache.set_other_actor_pk(verifier.pk);
    verifier.multisg_cache.set_other_actor_pk(prover.pk);

    let mut challenge_hashes_manager = ChallengeHashesManager::new();

    let (rpc, initial_fund_tx) = setup_client_and_fund_prover(
        WALLET_NAME,
        &prover.get_bitcoincore_rpc_address(),
        Amount::from_sat(100_000),
    );

    let secp = Secp256k1::new();
    // WTF is this actually
    let bisection_length = 10;
    let amt: u64 = 100_000;
    let fee: u64 = 500;
    let dust_limit: u64 = 546;

    let (equivocation_address, _) =
        generate_equivocation_address_and_info(&secp, &circuit, prover.pk, verifier.pk);

    let (response_second_address, _) = taproot_address_from_script_leaves(
        &secp,
        vec![
            generate_timelock_script(verifier.pk, 10),
            generate_2_of_2_script(prover.pk, verifier.pk),
        ],
    );

    let mut initial_fund_or_prev_response_tx = initial_fund_tx.clone().transaction().unwrap();

    let mut challenge_txs: Vec<Transaction> = vec![];
    let mut response_txs: Vec<Transaction> = vec![];

    // The verifier and provider here are creating the linked challenge - response transactions
    for i in 0..bisection_length {
        // Verifier creates the challenge hashes
        let (challenge_hashes, challenge_preimages) =
            challenge_hashes_manager.generate_challenge_hashes(circuit.gates.len());

        // Using the challenge hashes the verifier creates their challenge transaction which has a
        // leaf script for every
        let (challenge_address, _) =
            generate_challenge_address_and_info(&secp, &circuit, verifier.pk, &challenge_hashes);

        // Create a leaf script for every gate in the circuit that is unlockable by the
        // challenge hash. This is where the gate.create_response_script methods are called
        let (response_address, _) =
            generate_response_address_and_info(&secp, &circuit, prover.pk, &challenge_hashes);

        let challenge_tx = build_challenge_tx(
            &initial_fund_or_prev_response_tx,
            &challenge_address,
            &equivocation_address,
            amt,
            fee,
            dust_limit,
            i,
        );

        let response_tx = build_response_tx(
            &challenge_tx,
            &response_address,
            &response_second_address,
            amt,
            fee,
            dust_limit,
            i,
        );

        // Only on the first iteration we can skip the signing of the musig script for the
        // challenge transaction, thats because
        // the inputs to the first challenge transaction don't contain a musig script.
        if i != 0 {
            let sig = verifier.sign_tx_containing_musig(
                &challenge_tx,
                initial_fund_or_prev_response_tx.output.clone(),
            );

            // Prover saves signature so they can respond to a challenge later
            prover.add_signature(
                sig,
                &challenge_tx,
                initial_fund_or_prev_response_tx.output.clone(),
            );
        }

        let sig = prover.sign_tx_containing_musig(&response_tx, challenge_tx.output.clone());
        // Verifier saves signature so they can respond to a challenge later
        verifier.add_signature(sig, &response_tx, challenge_tx.output.clone());

        initial_fund_or_prev_response_tx = response_tx.clone();

        challenge_txs.push(challenge_tx);
        response_txs.push(response_tx);
    }

    let a1 = 633;
    let a2 = 15;
    let b1 = number_to_bool_array(a1, 64);
    let b2 = number_to_bool_array(a2, 64);

    let _o = circuit.evaluate(vec![b1, b2]);

    println!("Finished setup and sig exchange");

    let mut i = 0;

    let mut challenge_gate_num = 0;

    // Using into_iter here to consume the challenge_txs and response_txs vectors
    for (mut challenge_tx, mut response_tx) in
        challenge_txs.into_iter().zip(response_txs.into_iter())
    {
        if i == 0 {
            // Construct the witness data for the kickoff transaction, which is a simple spend of the
            // initial fund transaction
            let mut sighash_cache = SighashCache::new(&mut challenge_tx);

            let prevouts = vec![TxOut {
                script_pubkey: prover.address.script_pubkey(),
                value: Amount::from_sat(amt),
            }];
            let sig_hash = sighash_cache
                .taproot_key_spend_signature_hash(
                    0,
                    &bitcoin::sighash::Prevouts::All(&prevouts),
                    bitcoin::sighash::TapSighashType::Default,
                )
                .unwrap();

            // ISSUE: Some transactions are failing with "mandatory-script-verify-flag-failed (Invalid Schnorr signature)"
            let sig = prover.sign_with_tweak(sig_hash, None);
            let witness = sighash_cache.witness_mut(0).unwrap();
            witness.push(sig.as_ref());
        }

        let kickoff_txid = rpc
            .send_raw_transaction(&challenge_tx)
            .unwrap_or_else(|e| panic!("Failed to send raw transaction: {}", e));

        dbg!(&kickoff_txid);
        println!("hey");

        // TODO: Create witness data for response transaction

        // rpc.send_raw_transaction(&response_tx, None, None).unwrap();
        // if i != 0 {
        //     populate_challenge_tx_with_witness_data(
        //         &verifier,
        //         prover_pk,
        //         challenge_tx,
        //         &initial_fund_or_prev_response_tx.output,
        //         i,
        //     );
        // }

        // populate_response_tx_with_witness_data(
        //     &prover,
        //     verifier_pk,
        //     response_tx,
        //     &challenge_tx.output,
        //     i,
        // );

        i += 1;
    }
}
