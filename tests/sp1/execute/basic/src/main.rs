#![no_main]

sp1_zkvm::entrypoint!(main);
pub fn main() {
    // Read an input
    let n = sp1_zkvm::io::read::<u32>();
    let a = sp1_zkvm::io::read::<u16>() as u32;

    sp1_zkvm::io::commit(&((n + a) * 2));
}
