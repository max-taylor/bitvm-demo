pub mod challenge;
pub mod witness;

use std::str::FromStr;

use bitcoin::{
    key::Secp256k1,
    opcodes::all::{
        OP_BOOLOR, OP_CHECKSIG, OP_CHECKSIGVERIFY, OP_CSV, OP_DUP, OP_EQUAL, OP_EQUALVERIFY,
        OP_ROT, OP_SHA256, OP_VERIFY,
    },
    script::Builder,
    secp256k1::All,
    taproot::{TaprootBuilder, TaprootSpendInfo},
    Address, ScriptBuf, XOnlyPublicKey,
};

use crate::{
    circuit::{
        wire::{HashTuple, HashValue},
        BristolCircuit,
    },
    traits::gate::GateTrait,
};

pub fn add_bit_commitment_script(wire_bit_hashes: HashTuple, builder: Builder) -> Builder {
    builder
        .push_opcode(OP_SHA256)
        .push_opcode(OP_DUP)
        .push_slice(wire_bit_hashes.one)
        .push_opcode(OP_EQUAL)
        .push_opcode(OP_DUP)
        .push_opcode(OP_ROT)
        .push_slice(wire_bit_hashes.zero)
        .push_opcode(OP_EQUAL)
        .push_opcode(OP_BOOLOR)
        .push_opcode(OP_VERIFY)
}

pub fn generate_equivocation_address_and_info(
    secp: &Secp256k1<All>,
    circuit: &BristolCircuit,
    prover_pk: XOnlyPublicKey,
    verifier_pk: XOnlyPublicKey,
) -> (Address, TaprootSpendInfo) {
    // Creates an equivocation script for each wire in the circuit
    let mut scripts = circuit
        .wires
        .iter()
        .map(|wire_rcref| {
            // ! Generates the bitcoin script equivalent to equivocate
            generate_anti_contradiction_script(
                wire_rcref.lock().unwrap().get_hash_pair(),
                verifier_pk,
            )
        })
        .collect::<Vec<ScriptBuf>>();
    scripts.push(generate_timelock_script(prover_pk, 10));
    scripts.push(generate_2_of_2_script(prover_pk, verifier_pk));
    taproot_address_from_script_leaves(secp, scripts)
}

// This script is used by the verifier to equivocate the prover if they reveal both pre-images
pub fn generate_anti_contradiction_script(
    wire_bit_hashes: HashTuple,
    verifier_pk: XOnlyPublicKey,
) -> ScriptBuf {
    Builder::new()
        .push_opcode(OP_SHA256)
        .push_slice(wire_bit_hashes.zero)
        .push_opcode(OP_EQUALVERIFY)
        .push_opcode(OP_SHA256)
        .push_slice(wire_bit_hashes.one)
        .push_opcode(OP_EQUALVERIFY)
        .push_x_only_key(&verifier_pk)
        .push_opcode(OP_CHECKSIG)
        .into_script()
}

pub fn generate_timelock_script(actor_pk: XOnlyPublicKey, block_count: u32) -> ScriptBuf {
    Builder::new()
        .push_int(block_count as i64)
        .push_opcode(OP_CSV)
        .push_x_only_key(&actor_pk)
        .push_opcode(OP_CHECKSIG)
        .into_script()
}

pub fn generate_2_of_2_script(prover_pk: XOnlyPublicKey, verifier_pk: XOnlyPublicKey) -> ScriptBuf {
    Builder::new()
        .push_x_only_key(&prover_pk)
        .push_opcode(OP_CHECKSIGVERIFY)
        .push_x_only_key(&verifier_pk)
        .push_opcode(OP_CHECKSIG)
        .into_script()
}

pub fn taproot_address_from_script_leaves(
    secp: &Secp256k1<All>,
    scripts: Vec<ScriptBuf>,
) -> (Address, TaprootSpendInfo) {
    let n = scripts.len();
    assert!(n > 1, "more than one script is required");
    let m: u8 = ((n - 1).ilog2() + 1) as u8; // m = ceil(log(n))
    let k = 2_usize.pow(m.into()) - n;
    let taproot = (0..n).fold(TaprootBuilder::new(), |acc, i| {
        acc.add_leaf(m - ((i >= n - k) as u8), scripts[i].clone())
            .unwrap()
    });

    // An unspendable public key, which prevents the Public Key side of the taproot from being
    // spent
    // https://github.com/bitcoin/bips/blob/master/bip-0341.mediawiki#constructing-and-spending-taproot-outputs
    let internal_key = XOnlyPublicKey::from_str(
        "93c7378d96518a75448821c4f7c8f4bae7ce60f804d03d1f0628dd5dd0f5de51",
    )
    .unwrap();
    let tree_info = taproot.finalize(secp, internal_key).unwrap();
    let address = Address::p2tr(
        secp,
        internal_key,
        tree_info.merkle_root(),
        bitcoin::Network::Regtest,
    );
    (address, tree_info)
}

pub fn generate_challenge_address_and_info(
    secp: &Secp256k1<All>,
    circuit: &BristolCircuit,
    verifier_pk: XOnlyPublicKey,
    challenge_hashes: &Vec<HashValue>,
) -> (Address, TaprootSpendInfo) {
    assert_eq!(
        challenge_hashes.len(),
        circuit.gates.len(),
        "wrong number of challenge hashes"
    );
    let scripts = challenge_hashes
        .iter()
        .map(|x| generate_challenge_script(verifier_pk, x))
        .collect::<Vec<ScriptBuf>>();
    taproot_address_from_script_leaves(secp, scripts)
}

pub fn generate_challenge_script(
    verifier_pk: XOnlyPublicKey,
    challenge_hash: &HashValue,
) -> ScriptBuf {
    Builder::new()
        .push_opcode(OP_SHA256)
        .push_slice(challenge_hash)
        .push_opcode(OP_EQUALVERIFY)
        .push_x_only_key(&verifier_pk)
        .push_opcode(OP_CHECKSIG)
        .into_script()
}

pub fn generate_response_address_and_info(
    secp: &Secp256k1<All>,
    circuit: &BristolCircuit,
    prover_pk: XOnlyPublicKey,
    challenge_hashes: &Vec<HashValue>,
) -> (Address, TaprootSpendInfo) {
    assert_eq!(
        challenge_hashes.len(),
        circuit.gates.len(),
        "wrong number of challenge hashes"
    );
    let scripts = circuit
        .gates
        .iter()
        .zip(challenge_hashes.iter())
        .map(|(gate, hash)| generate_gate_response_script(gate, hash, prover_pk))
        .collect::<Vec<ScriptBuf>>();
    taproot_address_from_script_leaves(secp, scripts)
}

pub fn generate_gate_response_script(
    gate: &Box<dyn GateTrait>,
    challenge_hash: &HashValue,
    prover_pk: XOnlyPublicKey,
) -> ScriptBuf {
    Builder::from(
        gate.create_response_script(*challenge_hash)
            .as_bytes()
            .to_vec(),
    )
    .push_x_only_key(&prover_pk)
    .push_opcode(OP_CHECKSIG)
    .into_script()
}
