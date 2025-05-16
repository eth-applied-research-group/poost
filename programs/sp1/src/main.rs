#![no_main]

sp1_zkvm::entrypoint!(main);

fn main() {
    // Read the two inputs
    let value1: u32 = sp1_zkvm::io::read();
    let value2: u16 = sp1_zkvm::io::read();

    let result = (value1 as u64) * (value2 as u64) + (value1 as u64);

    // Write the result
    sp1_zkvm::io::commit(&result);
} 