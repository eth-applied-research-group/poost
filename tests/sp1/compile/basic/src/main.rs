#![no_main]

sp1_zkvm::entrypoint!(main);

pub fn main() {
    // Read an input
    let n = sp1_zkvm::io::read::<u32>();
    // Write n*2 to output
    sp1_zkvm::io::commit(&(n * 2));
} 