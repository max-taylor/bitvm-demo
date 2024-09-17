use circuit::BristolCircuit;

mod actor;
mod circuit;
mod prover;
mod traits;
mod transactions;
mod utils;

fn main() {
    let circuit = BristolCircuit::from_bristol("circuits/add.txt");
}
