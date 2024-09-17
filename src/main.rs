use bitcoin::key::Secp256k1;
use circuit::BristolCircuit;
use party::{prover::Prover, verifier::Verifier};
use traits::party::Party;
use transactions::generate_equivocation_address_and_info;

mod actor;
mod circuit;
mod party;
mod traits;
mod transactions;
mod utils;

fn main() {
    let circuit = BristolCircuit::from_bristol("circuits/add.txt");

    let prover = Prover::new();
    let verifier = Verifier::new();

    let secp = Secp256k1::new();
    let (equivocation_address, equivocation_taproot_info) = generate_equivocation_address_and_info(
        &secp,
        &circuit,
        prover.get_xonly_public_key(),
        verifier.get_xonly_public_key(),
    );
}
