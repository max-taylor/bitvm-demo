use bitcoin::{
    absolute::{Height, LockTime},
    Address, Amount, OutPoint, ScriptBuf, Transaction, TxIn, TxOut, Txid, Witness,
};

pub fn build_challenge_tx(
    prev_txid: &Txid,
    challenge_address: &Address,
    equivocation_address: &Address,
    amount: u64,
    fee: u64,
    dust_limit: u64,
    i: u64,
) -> Transaction {
    let inputs = if i == 0 {
        vec![TxIn {
            previous_output: OutPoint {
                txid: prev_txid.clone(),
                vout: 0, // TODO: THis may be incorrect, previously this was
                         // "initial_fund_tx.details[0].vout,"
            },
            script_sig: ScriptBuf::new(),
            sequence: bitcoin::transaction::Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: Witness::new(),
        }]
    } else {
        vec![
            TxIn {
                previous_output: OutPoint {
                    txid: prev_txid.clone(),
                    vout: 0,
                },
                script_sig: ScriptBuf::new(),
                sequence: bitcoin::transaction::Sequence::ENABLE_RBF_NO_LOCKTIME,
                witness: Witness::new(),
            },
            TxIn {
                previous_output: OutPoint {
                    txid: prev_txid.clone(),
                    vout: 1,
                },
                script_sig: ScriptBuf::new(),
                sequence: bitcoin::transaction::Sequence::ENABLE_RBF_NO_LOCKTIME,
                witness: Witness::new(),
            },
        ]
    };

    Transaction {
        version: bitcoin::transaction::Version::TWO,
        lock_time: LockTime::from(Height::MIN),
        input: inputs,
        output: vec![
            TxOut {
                script_pubkey: challenge_address.script_pubkey(),
                value: Amount::from_sat(dust_limit),
            },
            TxOut {
                script_pubkey: equivocation_address.script_pubkey(),
                value: Amount::from_sat(amount - (2 * i + 1) * (fee + dust_limit)),
            },
        ],
    }
}

pub fn build_response_tx(
    previous_challenge_tx: &Transaction,
    response_address: &Address,
    response_second_address: &Address,
    amt: u64,
    fee: u64,
    dust_limit: u64,
    i: u64,
) -> Transaction {
    Transaction {
        version: bitcoin::transaction::Version::TWO,
        lock_time: LockTime::from(Height::MIN),
        input: vec![
            TxIn {
                previous_output: OutPoint {
                    txid: previous_challenge_tx.txid(),
                    vout: 0,
                },
                script_sig: ScriptBuf::new(),
                sequence: bitcoin::transaction::Sequence::ENABLE_RBF_NO_LOCKTIME,
                witness: Witness::new(),
            },
            TxIn {
                previous_output: OutPoint {
                    txid: previous_challenge_tx.txid(),
                    vout: 1,
                },
                script_sig: ScriptBuf::new(),
                sequence: bitcoin::transaction::Sequence::ENABLE_RBF_NO_LOCKTIME,
                witness: Witness::new(),
            },
        ],
        output: vec![
            TxOut {
                script_pubkey: response_address.script_pubkey(),
                value: Amount::from_sat(dust_limit),
            },
            TxOut {
                script_pubkey: response_second_address.script_pubkey(),
                value: Amount::from_sat(amt - (2 * i + 2) * (fee + dust_limit)),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        actor::{Actor, ActorType},
        circuit::BristolCircuit,
        constants::WALLET_NAME,
        transactions::{
            generate_2_of_2_script, generate_challenge_address_and_info,
            generate_equivocation_address_and_info, generate_response_address_and_info,
            generate_timelock_script, taproot_address_from_script_leaves,
        },
        utils::{
            bitcoin_rpc::setup_client_and_fund_prover, challenge_hashes::ChallengeHashesManager,
            witness::fill_response_tx_with_witness_data,
        },
    };

    use super::*;
    use bitcoin::{key::Secp256k1, sighash::SighashCache, Amount, TxOut};
    use bitcoincore_rpc::{json::GetTransactionResult, Client, RpcApi};

    const INITIAL_FUND_AMOUNT: Amount = Amount::from_sat(100_000);
    const CHALLENGE_AMOUNT: u64 = 100_000;
    const FEE: u64 = 500;
    const DUST_LIMIT: u64 = 546;

    fn test_setup() -> (
        Client,
        GetTransactionResult,
        Actor,
        Actor,
        ChallengeHashesManager,
        Vec<TxOut>,
    ) {
        let mut prover = Actor::new(ActorType::Prover);
        let mut verifier = Actor::new(ActorType::Verifier);

        prover.multisg_cache.set_other_actor_pk(verifier.pk);
        verifier.multisg_cache.set_other_actor_pk(prover.pk);

        let (rpc, fund_tx) = setup_client_and_fund_prover(
            WALLET_NAME,
            &prover.get_bitcoincore_rpc_address(),
            INITIAL_FUND_AMOUNT,
        );

        let challenge_hash_manager = ChallengeHashesManager::new();

        let fund_tx_prevouts = vec![TxOut {
            script_pubkey: prover.address.script_pubkey(),
            value: INITIAL_FUND_AMOUNT,
        }];

        (
            rpc,
            fund_tx,
            prover,
            verifier,
            challenge_hash_manager,
            fund_tx_prevouts,
        )
    }

    #[test]
    fn test_generate_challenge_hash_script() {
        let (rpc, fund_tx, prover, verifier, mut challenge_hash_manager, fund_tx_prevouts) =
            test_setup();

        let secp = Secp256k1::new();
        let circuit = BristolCircuit::from_bristol("circuits/add.txt");
        let (equivocation_address, equivocation_taproot_info) =
            generate_equivocation_address_and_info(&secp, &circuit, prover.pk, verifier.pk);

        let (challenge_hashes, challenge_preimages) =
            challenge_hash_manager.generate_challenge_hashes(circuit.gates.len());

        let (challenge_address, challenge_taproot_info) =
            generate_challenge_address_and_info(&secp, &circuit, verifier.pk, &challenge_hashes);

        let mut challenge_tx = build_challenge_tx(
            &fund_tx.transaction().unwrap().txid(),
            &challenge_address,
            &equivocation_address,
            CHALLENGE_AMOUNT,
            FEE,
            DUST_LIMIT,
            0,
        );

        let mut sighash_cache = SighashCache::new(&mut challenge_tx);
        let sighash = sighash_cache
            .taproot_key_spend_signature_hash(
                0,
                &bitcoin::sighash::Prevouts::All(&fund_tx_prevouts),
                bitcoin::sighash::TapSighashType::Default,
            )
            .unwrap();
        let sig = prover.sign_with_tweak(sighash, None);
        let witness = sighash_cache.witness_mut(0).unwrap();
        witness.push(sig.as_ref());

        let txid = rpc
            .send_raw_transaction(&challenge_tx)
            .unwrap_or_else(|e| panic!("Failed to send challenge tx: {}", e));

        dbg!(txid);

        let (response_address, _) =
            generate_response_address_and_info(&secp, &circuit, prover.pk, &challenge_hashes);

        let (response_second_address, _) = taproot_address_from_script_leaves(
            &secp,
            vec![
                generate_timelock_script(verifier.pk, 10),
                generate_2_of_2_script(prover.pk, verifier.pk),
            ],
        );

        let mut response_tx = build_response_tx(
            &challenge_tx,
            &response_address,
            &response_second_address,
            CHALLENGE_AMOUNT,
            FEE,
            DUST_LIMIT,
            0,
        );

        let challenge_gate_num = 0;

        let prover_musig =
            prover.sign_tx_containing_musig(&response_tx, challenge_tx.output.clone());
        let verifier_musig =
            verifier.sign_tx_containing_musig(&response_tx, challenge_tx.output.clone());

        fill_response_tx_with_witness_data(
            &mut response_tx,
            &challenge_tx,
            &verifier,
            prover.pk,
            &challenge_hashes[challenge_gate_num as usize],
            &challenge_preimages[challenge_gate_num as usize],
            &challenge_taproot_info,
            &equivocation_taproot_info,
            &prover_musig,
            &verifier_musig,
        );

        let response_txid = rpc
            .send_raw_transaction(&response_tx)
            .unwrap_or_else(|e| panic!("Failed to send raw transaction: {}", e));

        let tx = rpc.get_raw_transaction(&response_txid, None).unwrap();
        dbg!(tx);
    }
}
