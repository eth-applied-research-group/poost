//! SP1 guest program

#![no_main]

extern crate alloc;

use reth_stateless::{ClientInput, validation::stateless_validation};

sp1_zkvm::entrypoint!(main);
/// Entry point.
pub fn main() {
    println!("cycle-tracker-report-start: read_input");
    let input = sp1_zkvm::io::read::<ClientInput>();
    println!("cycle-tracker-report-end: read_input");

    let chain_spec = &*reth_chainspec::HOODI;

    println!("cycle-tracker-report-start: validation");
    stateless_validation(input.block, input.witness, chain_spec.clone()).unwrap();
    println!("cycle-tracker-report-end: validation");
}
